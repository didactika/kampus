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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Enrolling oneself through a self instance: with its key when it has one, a group's key putting the
//! person in that group, and a request instead of an enrolment when the instance asks for approval. A
//! wrong key never says what was wrong, and attempts are limited.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, ErrorKind, Result};
use kampus_courses::GroupId;
use sqlx::Row;

use crate::permissions::SELF;
use crate::{Enrol, Instance, InstanceId, Joined, Method, Period, RequestId, sql};

/// How many wrong keys a person may give an instance in [`WINDOW`].
const ATTEMPTS: i64 = 5;

/// The window of the attempts, in minutes.
const WINDOW: i32 = 60;

/// The answer to a wrong or missing key.
fn wrong_key() -> Error {
    Error::validation(
        "enrol.key_wrong",
        vec![kampus_common::FieldError::new("key", "wrong")],
    )
}

impl Enrol {
    /// Enrols the actor through a self instance, with a key when it has one; needs `enrol.self` in the
    /// course. With an approval, the answer is a request that waits for it.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission or for a visitor, `validation` for a wrong key or an
    /// instance that is not a self one, `rate_limited` after too many wrong keys, `conflict` for a closed
    /// or full instance, an enrolment or a request that exists.
    pub async fn self_enrol(
        &self,
        actor: &Actor,
        instance: InstanceId,
        key: Option<&str>,
        message: Option<&str>,
        address: Option<&str>,
    ) -> Result<Joined> {
        let Actor::Principal(account) = actor else {
            return Err(Error::permission_denied("enrol.sign_in_required"));
        };
        let stored = self.instance(instance).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, SELF, scope).await?;
        if stored.method != Method::SelfEnrol {
            return Err(Error::validation(
                "enrol.method_not_self",
                vec![kampus_common::FieldError::new("instance", "self")],
            ));
        }
        self.throttle(instance, *account).await?;
        let group = if stored.has_key {
            let found = match key {
                Some(key) => self.matching_key(instance, key).await?,
                None => None,
            };
            self.record_attempt(instance, *account, address, found.is_some())
                .await?;
            match found {
                Some(group) => group,
                None => return Err(wrong_key()),
            }
        } else {
            None
        };
        if stored.requires_approval {
            let request = self.open_request(&stored, *account, group, message).await?;
            return Ok(Joined::Requested { request });
        }
        let enrolment = self
            .create_enrolment(&stored, *account, Period::default(), Some(*account))
            .await?;
        if let Some(group) = group {
            self.join_group(instance, group, *account).await?;
        }
        Ok(Joined::Enrolled { enrolment })
    }

    /// Refuses a person who gave too many wrong keys to an instance lately.
    async fn throttle(&self, instance: InstanceId, account: PrincipalId) -> Result<()> {
        let failed: i64 = sqlx::query_scalar(sql(format!(
            "select count(*) from {} where instance_id = $1 and account_id = $2 and not succeeded
             and at > now() - make_interval(mins => $3)",
            self.table("self_attempt")
        )))
        .bind(instance.as_uuid())
        .bind(account.as_uuid())
        .bind(WINDOW)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if failed >= ATTEMPTS {
            return Err(Error::new(ErrorKind::RateLimited, "enrol.key_throttled")
                .with_param("retry_after", (i64::from(WINDOW) * 60).to_string()));
        }
        Ok(())
    }

    /// The key a given key matches: `Some(None)` for the instance's own, `Some(Some(group))` for a group's.
    async fn matching_key(&self, instance: InstanceId, key: &str) -> Result<Option<Option<GroupId>>> {
        let rows = sqlx::query(sql(format!(
            "select group_id, hash from {} where instance_id = $1 order by group_id nulls first",
            self.table("self_key")
        )))
        .bind(instance.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        for row in rows {
            let hash: String = row.get("hash");
            if self.hasher.verify(key, Some(&hash)).await? {
                return Ok(Some(
                    row.get::<Option<uuid::Uuid>, _>("group_id")
                        .map(GroupId::from_uuid),
                ));
            }
        }
        Ok(None)
    }

    /// Records an attempt with a key.
    async fn record_attempt(
        &self,
        instance: InstanceId,
        account: PrincipalId,
        address: Option<&str>,
        succeeded: bool,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, instance_id, account_id, address, succeeded) values ($1, $2, $3, $4, $5)",
            self.table("self_attempt")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(instance.as_uuid())
        .bind(account.as_uuid())
        .bind(address)
        .bind(succeeded)
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Opens a request to join, once per instance and person while it waits.
    async fn open_request(
        &self,
        instance: &Instance,
        account: PrincipalId,
        group: Option<GroupId>,
        message: Option<&str>,
    ) -> Result<RequestId> {
        if self.is_enrolled(instance.id, account).await? {
            return Err(Error::conflict("enrol.already_enrolled"));
        }
        let id = RequestId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, instance_id, account_id, message, group_id) values ($1, $2, $3, $4, $5)",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .bind(instance.id.as_uuid())
        .bind(account.as_uuid())
        .bind(message.map(str::trim).filter(|m| !m.is_empty()))
        .bind(group.map(|g| g.as_uuid()))
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.unique_violation" {
                Error::conflict("enrol.request_pending")
            } else {
                e
            }
        })?;
        Ok(id)
    }

    /// Whether the account has an enrolment through the instance, whatever its status.
    pub(crate) async fn is_enrolled(&self, instance: InstanceId, account: PrincipalId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where instance_id = $1 and account_id = $2)",
            self.table("enrolment")
        )))
        .bind(instance.as_uuid())
        .bind(account.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }
}
