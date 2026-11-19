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

//! Requests to join: they wait for someone with `enrol.request.decide`; approving one creates the
//! enrolment, and every decision is kept with who made it.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, Result};
use kampus_courses::{CourseId, GroupId};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::enrolment::principal;
use super::instant;
use crate::permissions::REQUEST_DECIDE;
use crate::{Enrol, InstanceId, Period, Request, RequestId, RequestState, sql};

/// A request from its row.
fn request(row: &PgRow) -> Request {
    Request {
        id: RequestId::from_uuid(row.get("id")),
        instance: InstanceId::from_uuid(row.get("instance_id")),
        account: PrincipalId::from_uuid(row.get("account_id")),
        message: row.get("message"),
        group: row
            .get::<Option<uuid::Uuid>, _>("group_id")
            .map(GroupId::from_uuid),
        state: RequestState::read(row.get::<String, _>("state").as_str()),
        created_at: instant(row, "created_at").unwrap_or_else(|| {
            kampus_common::Timestamp::from_offset_date_time(time::OffsetDateTime::UNIX_EPOCH)
        }),
    }
}

impl Enrol {
    /// The requests of a course that wait for a decision, oldest first; needs `enrol.request.decide`.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn pending_requests(&self, actor: &Actor, course: CourseId) -> Result<Vec<Request>> {
        let scope = self.courses.course(course).await?.scope;
        self.access.require(actor, REQUEST_DECIDE, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select r.id, r.instance_id, r.account_id, r.message, r.group_id, r.state, r.created_at
             from {} r join {} i on i.id = r.instance_id where i.course_id = $1 and r.state = 'pending'
             order by r.created_at, r.id",
            self.table("request"),
            self.table("instance")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(request).collect())
    }

    /// Approves or rejects a request; approving creates the enrolment, with the group of the key given.
    /// Needs `enrol.request.decide` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `conflict` (`enrol.request_decided`) for a request that
    /// does not wait any more, and the errors of enrolling.
    pub async fn decide_request(
        &self,
        actor: &Actor,
        id: RequestId,
        approve: bool,
        reason: Option<&str>,
    ) -> Result<Request> {
        let stored = self.request(id).await?;
        let instance = self.instance(stored.instance).await?;
        let scope = self.courses.course(instance.course).await?.scope;
        self.access.require(actor, REQUEST_DECIDE, scope).await?;
        if stored.state != RequestState::Pending {
            return Err(Error::conflict("enrol.request_decided"));
        }
        if approve {
            self.create_enrolment(&instance, stored.account, Period::default(), principal(actor))
                .await?;
            if let Some(group) = stored.group {
                self.join_group(instance.id, group, stored.account).await?;
            }
        }
        let state = if approve { "approved" } else { "rejected" };
        let changed = sqlx::query(sql(format!(
            "update {} set state = $2, decided_by = $3, decided_at = now(), reason = $4
             where id = $1 and state = 'pending'",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .bind(state)
        .bind(principal(actor).map(|p| p.as_uuid()))
        .bind(reason.map(str::trim).filter(|r| !r.is_empty()))
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            return Err(Error::conflict("enrol.request_decided"));
        }
        self.request(id).await
    }

    /// Withdraws the actor's own request while it waits.
    ///
    /// # Errors
    ///
    /// `not_found` for a request of someone else, `conflict` for one already decided.
    pub async fn withdraw_request(&self, actor: &Actor, id: RequestId) -> Result<()> {
        let stored = self.request(id).await?;
        if principal(actor) != Some(stored.account) {
            return Err(Error::not_found("enrol.request_not_found"));
        }
        let changed = sqlx::query(sql(format!(
            "update {} set state = 'withdrawn', decided_at = now() where id = $1 and state = 'pending'",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            return Err(Error::conflict("enrol.request_decided"));
        }
        Ok(())
    }

    /// A request.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown request.
    pub async fn request(&self, id: RequestId) -> Result<Request> {
        sqlx::query(sql(format!(
            "select id, instance_id, account_id, message, group_id, state, created_at from {} where id = $1",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .map(|row| request(&row))
        .ok_or_else(|| Error::not_found("enrol.request_not_found"))
    }
}
