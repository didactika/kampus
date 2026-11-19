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

//! Expiry: when an enrolment reaches its end, its instance's rule says what happens, and the outcome is
//! recorded. Records and progress are never touched.

use kampus_common::Result;
use sqlx::Row;

use crate::{Enrol, EnrolmentId, Expired, ExpiryAction, sql};

impl Enrol {
    /// Applies the expiry rule to every enrolment whose end has passed and that was not handled yet: it is
    /// suspended, removed, or kept without its role and without an end.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn run_expiry(&self) -> Result<Expired> {
        let rows = sqlx::query(sql(format!(
            "select e.id, e.instance_id, e.account_id, i.expiry_action from {} e join {} i on i.id = e.instance_id
             where e.ends_at <= now() and e.expired_at is null and e.status = 'active' order by e.ends_at, e.id",
            self.table("enrolment"),
            self.table("instance")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut expired = Expired::default();
        for row in rows {
            let id = EnrolmentId::from_uuid(row.get("id"));
            let action = ExpiryAction::read(row.get::<String, _>("expiry_action").as_str());
            let mut tx = self.db.begin().await?;
            sqlx::query(sql(format!(
                "insert into {} (id, enrolment_id, instance_id, account_id, action) values ($1, $2, $3, $4, $5)",
                self.table("expiry_outcome")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(id.as_uuid())
            .bind(row.get::<uuid::Uuid, _>("instance_id"))
            .bind(row.get::<uuid::Uuid, _>("account_id"))
            .bind(action.as_str())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            if action != ExpiryAction::Remove {
                self.revoke_grants_in(&mut tx, id).await?;
                let change = if action == ExpiryAction::Suspend {
                    "status = 'suspended', expired_at = now()"
                } else {
                    "ends_at = null, expired_at = now()"
                };
                sqlx::query(sql(format!(
                    "update {} set {change}, version = version + 1 where id = $1",
                    self.table("enrolment")
                )))
                .bind(id.as_uuid())
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
            tx.commit().await.map_err(kampus_db::classify)?;
            match action {
                ExpiryAction::Suspend => expired.suspended += 1,
                ExpiryAction::KeepWithoutRole => expired.kept_without_role += 1,
                ExpiryAction::Remove => {
                    let enrolment = self.enrolment(id).await?;
                    self.delete_enrolment(&enrolment).await?;
                    expired.removed += 1;
                }
            }
        }
        Ok(expired)
    }
}
