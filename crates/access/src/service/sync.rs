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

//! Synchronises what modules declare (DEC-015): permissions and role templates. A permission no module
//! declares any more is retired, never deleted, so references keep working (RN-021).

use kampus_common::Result;

use crate::model::{PermissionKind, RoleTemplateDeclaration};
use crate::{Access, PermissionDeclaration, sql};

impl Access {
    /// Brings what the given modules declare into the database: their permissions, and their values for
    /// each role template, merged.
    ///
    /// # Errors
    ///
    /// As [`Access::sync`].
    pub async fn sync_modules(&self, modules: &[&kampus_module::Module]) -> Result<()> {
        let permissions: Vec<(&str, &[PermissionDeclaration])> =
            modules.iter().map(|m| (m.name, m.permissions)).collect();
        let templates: Vec<RoleTemplateDeclaration> =
            modules.iter().flat_map(|m| m.templates.iter().copied()).collect();
        self.sync(&permissions, &templates).await
    }

    /// Brings the declared permissions and role templates into the database, in one transaction.
    ///
    /// # Errors
    ///
    /// A `validation` error when a template gives a value to an undeclared permission, an `unavailable`
    /// error when the database fails.
    pub async fn sync(
        &self,
        permissions: &[(&str, &[PermissionDeclaration])],
        templates: &[RoleTemplateDeclaration],
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut codes = Vec::new();
        for (module, declarations) in permissions {
            for permission in *declarations {
                codes.push(permission.code.to_owned());
                let kind = match permission.kind {
                    PermissionKind::Read => "read",
                    PermissionKind::Write => "write",
                };
                let risks: Vec<&str> = permission.risks.iter().map(|r| r.as_str()).collect();
                sqlx::query(sql(format!(
                    "insert into {} (code, module, kind, risks, lowest_kind, retired_at) values ($1, $2, $3, $4, $5, null)
                     on conflict (code) do update set module = excluded.module, kind = excluded.kind, risks = excluded.risks,
                     lowest_kind = excluded.lowest_kind, retired_at = null",
                    self.table("permission")
                )))
                .bind(permission.code)
                .bind(*module)
                .bind(kind)
                .bind(&risks)
                .bind(permission.lowest_kind.as_str())
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = $2 where not (code = any($1)) and retired_at is null",
            self.table("permission")
        )))
        .bind(&codes)
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;

        for template in templates {
            let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
                "insert into {} (id, code, version) values ($1, $2, $3)
                 on conflict (code, version) do update set code = excluded.code returning id",
                self.table("role_template")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(template.code)
            .bind(i32::try_from(template.version).unwrap_or(i32::MAX))
            .fetch_one(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            for (permission, value) in template.values {
                sqlx::query(sql(format!(
                    "insert into {} (template_id, permission_code, value) values ($1, $2, $3)
                     on conflict (template_id, permission_code) do update set value = excluded.value",
                    self.table("role_template_value")
                )))
                .bind(id)
                .bind(*permission)
                .bind(value.as_str())
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        tx.commit().await.map_err(kampus_db::classify)
    }
}
