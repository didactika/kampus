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

//! System group instances derive their enrolments from the group: a sync adds the members who are not
//! enrolled and removes the enrolments of those who left, and touches nothing else.

use std::collections::BTreeSet;

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, Result};
use sqlx::Row;

use crate::permissions::INSTANCE_MANAGE;
use crate::{Enrol, InstanceId, Method, Period, Synced, sql};

impl Enrol {
    /// Brings the enrolments of a system group instance in step with its group; needs
    /// `enrol.instance.manage` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an instance of another method.
    pub async fn sync_instance(&self, actor: &Actor, id: InstanceId) -> Result<Synced> {
        let stored = self.instance(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, INSTANCE_MANAGE, scope).await?;
        self.sync(id).await
    }

    /// Brings every system group instance in step with its group, for the scheduled run.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; an instance that fails stops the run.
    pub async fn sync_all(&self) -> Result<Synced> {
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select instance_id from {} order by instance_id",
            self.table("linked_group")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut total = Synced::default();
        for id in ids {
            let synced = self.sync(InstanceId::from_uuid(id)).await?;
            total.added += synced.added;
            total.removed += synced.removed;
        }
        Ok(total)
    }

    /// Brings in step every instance that follows a system group, when its members changed.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sync_group(&self, group: kampus_users::GroupId) -> Result<Synced> {
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select instance_id from {} where system_group_id = $1 order by instance_id",
            self.table("linked_group")
        )))
        .bind(group.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut total = Synced::default();
        for id in ids {
            let synced = self.sync(InstanceId::from_uuid(id)).await?;
            total.added += synced.added;
            total.removed += synced.removed;
        }
        Ok(total)
    }

    /// Syncs one instance.
    async fn sync(&self, id: InstanceId) -> Result<Synced> {
        let instance = self.instance(id).await?;
        let Some(source) = instance.source.filter(|_| instance.method == Method::SystemGroup) else {
            return Err(Error::validation(
                "enrol.method_not_system_group",
                vec![kampus_common::FieldError::new("instance", "system_group")],
            ));
        };
        let members: BTreeSet<PrincipalId> = self.users.member_accounts(source).await?.into_iter().collect();
        let rows = sqlx::query(sql(format!(
            "select e.id, e.account_id from {} e where e.instance_id = $1",
            self.table("enrolment")
        )))
        .bind(id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut synced = Synced::default();
        let mut enrolled = BTreeSet::new();
        for row in &rows {
            let account = PrincipalId::from_uuid(row.get("account_id"));
            enrolled.insert(account);
            if !members.contains(&account) {
                let enrolment = self
                    .enrolment(crate::EnrolmentId::from_uuid(row.get("id")))
                    .await?;
                self.delete_enrolment(&enrolment).await?;
                synced.removed += 1;
            }
        }
        for account in members.difference(&enrolled) {
            match self
                .create_enrolment(&instance, *account, Period::default(), None)
                .await
            {
                Ok(_) => synced.added += 1,
                Err(e) if e.code() == "enrol.instance_full" || e.code() == "enrol.instance_closed" => break,
                Err(e) => return Err(e),
            }
        }
        Ok(synced)
    }
}
