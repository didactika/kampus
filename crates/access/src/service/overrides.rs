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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Overrides (UC-002): how a role's value for a permission changes in a scope and its descendants
//! (RN-012, RN-014, RN-015, RN-008).

use kampus_common::{Error, FieldError, Result};
use serde_json::json;
use sqlx::Row;

use crate::permissions::{OVERRIDE_RISKY, OVERRIDE_UPDATE};
use crate::service::role::principal;
use crate::{Access, Actor, AllowanceKind, RoleId, ScopeId, ScopeKind, Value, sql};

impl Access {
    /// Sets the value of a role for a permission in a scope and its descendants; `None` inherits again,
    /// which deletes the override (RN-014, RN-012, RN-015, RN-008).
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permissions or the allowance, `validation` below the permission's
    /// lowest kind, `conflict` for RN-008.
    pub async fn set_override(
        &self,
        actor: &Actor,
        scope: ScopeId,
        role: RoleId,
        permission: &str,
        value: Option<Value>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!("select lowest_kind, cardinality(risks) > 0 as risky from {} where code = $1 and retired_at is null", self.table("permission"))))
            .bind(permission)
            .fetch_optional(&mut *tx)
            .await
            .map_err(kampus_db::classify)?
            .ok_or_else(|| Error::not_found("access.permission_not_found").with_param("permission", permission))?;
        let lowest: ScopeKind = row.get::<String, _>("lowest_kind").parse()?;
        let path = self.path(&mut tx, scope).await?;
        if path[0].kind.rank() > lowest.rank() {
            return Err(Error::validation(
                "access.override_below_lowest_kind",
                vec![FieldError::new("scope", "kind")],
            )
            .with_param("lowest_kind", lowest.as_str()));
        }
        self.require_with_allowance(
            &mut tx,
            actor,
            OVERRIDE_UPDATE,
            scope,
            role,
            AllowanceKind::Override,
        )
        .await?;
        if row.get::<bool, _>("risky")
            && !self
                .evaluate(&mut tx, actor, OVERRIDE_RISKY, scope, false)
                .await?
                .allowed
        {
            return Err(Error::permission_denied("access.denied").with_param("permission", OVERRIDE_RISKY));
        }
        let campus = self.campus_of(&mut tx, scope).await?;
        let was_managed = self.managed_before(&mut tx, campus).await?;
        let table = self.table("permission_override");
        let before: Option<String> = sqlx::query_scalar(sql(format!(
            "select value from {table} where scope_id = $1 and role_id = $2 and permission_code = $3"
        )))
        .bind(scope.as_uuid())
        .bind(role.as_uuid())
        .bind(permission)
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        match value {
            Some(value) => sqlx::query(sql(format!(
                "insert into {table} (scope_id, role_id, permission_code, value) values ($1, $2, $3, $4)
                 on conflict (scope_id, role_id, permission_code) do update set value = excluded.value, version = {table}.version + 1"
            )))
            .bind(scope.as_uuid())
            .bind(role.as_uuid())
            .bind(permission)
            .bind(value.as_str()),
            None => sqlx::query(sql(format!("delete from {table} where scope_id = $1 and role_id = $2 and permission_code = $3")))
                .bind(scope.as_uuid())
                .bind(role.as_uuid())
                .bind(permission),
        }
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.ensure_still_managed(&mut tx, campus, was_managed).await?;
        let key = format!("{scope}/{role}/{permission}");
        self.record(
            &mut tx,
            principal(actor),
            "permission_override",
            &key,
            before.map(|v| json!(v)),
            value.map(|v| json!(v)),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
