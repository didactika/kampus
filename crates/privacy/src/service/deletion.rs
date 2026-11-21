// ------------------------------------------------------------------------------
//
//     _  __    _    __  __ ____  _   _ ____
//    | |/ /   / \  |  \/  |  _ \| | | / ___|
//    | ' /   / _ \ | |\/| | |_) | | | \___ \
//    | . \  / ___ \| |  | |  __/| |_| |___) |
//    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
//
//    Knowledge Academic Modular Platform for Universal Study
//
// ------------------------------------------------------------------------------
//
// This file is part of Kampus - https://github.com/didactika/kampus
//
// Kampus is free software: you can redistribute it and/or modify it under the terms of the
// GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with Kampus.
// If not, see <https://www.gnu.org/licenses/>.
//
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Deletion requests: a person asks for the erasure of their personal data, someone who holds
//! `privacy.request.approve` approves or rejects it, every module erases its own data, and the account is
//! left as a tombstone. A legal hold keeps the request waiting.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result};
use serde_json::json;
use sqlx::Row;

use super::request::{Opening, person};
use crate::permissions::{REQUEST_APPROVE, REQUEST_MANAGE};
use crate::{Privacy, Request, RequestId, events, sql};

/// The longest reason a deletion may give, in characters.
const REASON_LIMIT: usize = 2000;

impl Privacy {
    /// Whether the actor may approve deletions in the campus of an account.
    async fn may_approve(&self, actor: &Actor, account: PrincipalId) -> Result<bool> {
        let campus = self.access.principal_campus(account).await?;
        Ok(self.access.check(actor, REQUEST_APPROVE, campus).await?.allowed)
    }

    /// Asks for the erasure of the personal data of an account: one's own, or another's with
    /// `privacy.request.manage`. It waits for approval, unless whoever asks may approve it.
    ///
    /// # Errors
    ///
    /// `permission_denied` for someone else's account without the permission, `validation` for a missing or
    /// too long reason, `conflict` (`privacy.deletion_open`) when one is already open for the account.
    pub async fn request_deletion(
        &self,
        actor: &Actor,
        account: PrincipalId,
        reason: &str,
    ) -> Result<Request> {
        let by = person(actor)?;
        if !self.may_handle(actor, account).await? {
            return Err(Error::permission_denied("access.denied").with_param("permission", REQUEST_MANAGE));
        }
        let reason = reason.trim();
        if reason.is_empty() || reason.chars().count() > REASON_LIMIT {
            return Err(Error::validation(
                "privacy.reason_invalid",
                vec![FieldError::new("reason", "required")],
            ));
        }
        let open: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where account_id = $1 and kind = 'deletion'
                 and state in ('pending', 'awaiting_approval', 'held'))",
            self.table("request")
        )))
        .bind(account.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if open {
            return Err(Error::conflict("privacy.deletion_open"));
        }
        let approved = self.may_approve(actor, account).await?;
        self.open_request(
            actor,
            account,
            Opening {
                kind: "deletion",
                state: if approved { "pending" } else { "awaiting_approval" },
                reason: Some(reason),
                approved_by: approved.then_some(by),
            },
        )
        .await
    }

    /// The deletions waiting for approval in the campus of the actor, oldest first.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `privacy.request.approve`.
    pub async fn awaiting_approval(&self, actor: &Actor) -> Result<Vec<Request>> {
        let me = person(actor)?;
        let campus = self.access.principal_campus(me).await?;
        if !self.access.check(actor, REQUEST_APPROVE, campus).await?.allowed {
            return Err(Error::permission_denied("access.denied").with_param("permission", REQUEST_APPROVE));
        }
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select r.id from {r} r join {p} p on p.id = r.account_id
             where r.state = 'awaiting_approval' and p.campus_id = $1 order by r.created_at, r.id limit 100",
            r = self.table("request"),
            p = self.principals()
        )))
        .bind(campus.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut found = Vec::new();
        for id in ids {
            found.push(self.request(actor, RequestId::from_uuid(id)).await?);
        }
        Ok(found)
    }

    /// Approves or rejects a deletion that waits for it. An approved deletion starts: modules erase their
    /// data and the account becomes a tombstone.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown request, `permission_denied` without `privacy.request.approve` in the
    /// campus of the account, `conflict` (`privacy.request_not_waiting`) when nothing waits for a decision.
    pub async fn decide(&self, actor: &Actor, id: RequestId, approve: bool) -> Result<Request> {
        let by = person(actor)?;
        let row = sqlx::query(sql(format!(
            "select account_id from {} where id = $1 and kind = 'deletion'",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("privacy.request_not_found"))?;
        let account = PrincipalId::from_uuid(row.get("account_id"));
        if !self.may_approve(actor, account).await? {
            return Err(Error::permission_denied("access.denied").with_param("permission", REQUEST_APPROVE));
        }
        let mut tx = self.db.begin().await?;
        let changed = sqlx::query(sql(format!(
            "update {} set state = $2, approved_by = $3, decided_at = now()
             where id = $1 and state = 'awaiting_approval'",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .bind(if approve { "pending" } else { "rejected" })
        .bind(approve.then(|| by.as_uuid()))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            return Err(Error::conflict("privacy.request_not_waiting"));
        }
        let mut event = kampus_events::Publish::new(
            events::REQUEST_DECIDED,
            Some(id.as_uuid()),
            json!({ "request": id, "account": account, "approved": approve }),
        );
        event.actor = Some(by.as_uuid());
        event.summary = Some(format!("{} {id}", events::REQUEST_DECIDED));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        if approve {
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(&mut tx, "privacy.run", &json!({}), None)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.request(actor, id).await
    }
}
