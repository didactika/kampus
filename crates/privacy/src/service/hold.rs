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

//! Legal holds: while one is active for an account, or for its whole campus, a deletion waits.

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::request::person;
use crate::permissions::HOLD_MANAGE;
use crate::{Hold, HoldId, Privacy, events, sql};

/// A hold from its row.
fn hold(row: &PgRow) -> Hold {
    let stamp = |column: &str| {
        row.get::<Option<time::OffsetDateTime>, _>(column)
            .map(Timestamp::from_offset_date_time)
    };
    Hold {
        id: HoldId::from_uuid(row.get("id")),
        account: row
            .get::<Option<uuid::Uuid>, _>("account_id")
            .map(PrincipalId::from_uuid),
        reason: row.get("reason"),
        until: stamp("until"),
        created_at: Timestamp::from_offset_date_time(row.get("created_at")),
        released_at: stamp("released_at"),
    }
}

impl Privacy {
    /// Checks that the actor holds `privacy.hold.manage` in a campus.
    async fn may_hold(&self, actor: &Actor, campus: ScopeId) -> Result<()> {
        if self.access.check(actor, HOLD_MANAGE, campus).await?.allowed {
            Ok(())
        } else {
            Err(Error::permission_denied("access.denied").with_param("permission", HOLD_MANAGE))
        }
    }

    /// Places a hold on the deletions of an account, or of the whole campus of the actor when no account is
    /// given.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `privacy.hold.manage`, `validation` for an empty reason.
    pub async fn place_hold(
        &self,
        actor: &Actor,
        account: Option<PrincipalId>,
        reason: &str,
        until: Option<Timestamp>,
    ) -> Result<Hold> {
        let by = person(actor)?;
        let campus = self.access.principal_campus(account.unwrap_or(by)).await?;
        self.may_hold(actor, campus).await?;
        if reason.trim().is_empty() {
            return Err(Error::validation(
                "privacy.reason_invalid",
                vec![FieldError::new("reason", "required")],
            ));
        }
        let id = HoldId::generate();
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, campus_id, reason, until, created_by)
             values ($1, $2, $3, $4, $5, $6)",
            self.table("legal_hold")
        )))
        .bind(id.as_uuid())
        .bind(account.map(|a| a.as_uuid()))
        .bind(account.is_none().then(|| campus.as_uuid()))
        .bind(reason.trim())
        .bind(until.map(|t| t.as_offset_date_time()))
        .bind(by.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut event = kampus_events::Publish::new(
            events::HOLD_PLACED,
            Some(id.as_uuid()),
            json!({ "hold": id, "account": account, "campus": campus }),
        );
        event.actor = Some(by.as_uuid());
        event.summary = Some(format!("{} {id}", events::HOLD_PLACED));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.hold(actor, id).await
    }

    /// A hold of the campus of the actor.
    async fn hold(&self, actor: &Actor, id: HoldId) -> Result<Hold> {
        self.holds_where(actor, "and h.id = $2", Some(id.as_uuid()))
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| Error::not_found("privacy.hold_not_found"))
    }

    /// The holds of the campus of the actor, active ones first.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `privacy.hold.manage`.
    pub async fn holds(&self, actor: &Actor) -> Result<Vec<Hold>> {
        self.holds_where(actor, "", None).await
    }

    /// The holds of the campus of the actor, narrowed by a condition with at most one bound id (`$2`).
    async fn holds_where(&self, actor: &Actor, extra: &str, id: Option<uuid::Uuid>) -> Result<Vec<Hold>> {
        let me = person(actor)?;
        let campus = self.access.principal_campus(me).await?;
        self.may_hold(actor, campus).await?;
        let rows = sqlx::query(sql(format!(
            "select h.id, h.account_id, h.reason, h.until, h.created_at, h.released_at from {h} h
             where (h.campus_id = $1 or h.account_id in (select id from {p} where campus_id = $1)) {extra}
             order by (h.released_at is null) desc, h.created_at desc, h.id limit 200",
            h = self.table("legal_hold"),
            p = self.principals()
        )))
        .bind(campus.as_uuid())
        .bind(id)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(hold).collect())
    }

    /// Releases a hold; releasing one already released changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found` for a hold of another campus, `permission_denied` without `privacy.hold.manage`.
    pub async fn release_hold(&self, actor: &Actor, id: HoldId) -> Result<Hold> {
        let by = person(actor)?;
        self.hold(actor, id).await?;
        let mut tx = self.db.begin().await?;
        let released = sqlx::query(sql(format!(
            "update {} set released_at = now() where id = $1 and released_at is null",
            self.table("legal_hold")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if released > 0 {
            let mut event =
                kampus_events::Publish::new(events::HOLD_RELEASED, Some(id.as_uuid()), json!({ "hold": id }));
            event.actor = Some(by.as_uuid());
            event.summary = Some(format!("{} {id}", events::HOLD_RELEASED));
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, event)
                .await?;
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(&mut tx, "privacy.run", &json!({}), None)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.hold(actor, id).await
    }
}
