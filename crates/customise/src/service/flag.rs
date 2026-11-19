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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Feature flags: features modules offer, on or off per scope, resolved from the nearest scope up with
//! locks. A feature turned off hides what it offers there; its data stays.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use serde_json::json;
use sqlx::Row;

use super::{SetIn, choose, principal};
use crate::permissions::FLAG_MANAGE;
use crate::{Customise, Flag, Origin, sql};

impl Customise {
    /// Whether a feature is on in a scope, with where the value came from.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.flag_unknown`) for a flag no module declares.
    pub async fn flag(&self, code: &str, scope: ScopeId) -> Result<Flag> {
        self.flags_where(scope, Some(code))
            .await?
            .pop()
            .ok_or_else(|| Error::not_found("customise.flag_unknown").with_param("code", code.to_owned()))
    }

    /// Every feature with its value in a scope, by code.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn flags(&self, scope: ScopeId) -> Result<Vec<Flag>> {
        self.flags_where(scope, None).await
    }

    /// Fails unless a feature is on in a scope, for a module that guards what the feature offers.
    ///
    /// # Errors
    ///
    /// `conflict` (`customise.feature_off`) when it is off there.
    pub async fn require_flag(&self, code: &str, scope: ScopeId) -> Result<()> {
        if self.flag(code, scope).await?.enabled {
            Ok(())
        } else {
            Err(Error::conflict("customise.feature_off").with_param("code", code.to_owned()))
        }
    }

    /// Turns a feature on or off in a scope, locked or not; needs `customise.flag.manage` there.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown flag, `permission_denied` without the permission, `conflict`
    /// (`customise.locked_above`) below a scope that locks it.
    pub async fn set_flag(
        &self,
        actor: &Actor,
        code: &str,
        scope: ScopeId,
        enabled: bool,
        locked: bool,
    ) -> Result<Flag> {
        self.access.require(actor, FLAG_MANAGE, scope).await?;
        let current = self.flag(code, scope).await?;
        if let Origin::Scope {
            scope: at,
            locked: true,
        } = current.origin
            && at != scope
        {
            return Err(Error::conflict("customise.locked_above"));
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (flag_code, scope_id, enabled, locked) values ($1, $2, $3, $4)
             on conflict (flag_code, scope_id) do update set enabled = excluded.enabled, locked = excluded.locked",
            self.table("flag_value")
        )))
        .bind(code)
        .bind(scope.as_uuid())
        .bind(enabled)
        .bind(locked)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.record_in(
            &mut tx,
            "flag",
            code,
            scope,
            principal(actor),
            (
                Some(json!({ "enabled": current.enabled })),
                Some(json!({ "enabled": enabled, "locked": locked })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.flag(code, scope).await
    }

    /// The flags a code selects, or all, resolved in a scope.
    async fn flags_where(&self, scope: ScopeId, code: Option<&str>) -> Result<Vec<Flag>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let depths = self.depths(&mut conn, scope).await?;
        let ids: Vec<uuid::Uuid> = depths.keys().copied().collect();
        let rows = sqlx::query(sql(format!(
            "select f.code, f.default_value, v.scope_id, v.enabled, v.locked from {} f
             left join {} v on v.flag_code = f.code and v.scope_id = any($1)
             where f.retired_at is null and ($2::text is null or f.code = $2) order by f.code",
            self.table("flag"),
            self.table("flag_value")
        )))
        .bind(&ids)
        .bind(code)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut flags: Vec<(String, bool, Vec<SetIn<bool>>)> = Vec::new();
        for row in &rows {
            let code: String = row.get("code");
            if flags.last().is_none_or(|(c, _, _)| *c != code) {
                flags.push((code, row.get("default_value"), Vec::new()));
            }
            if let (Some(id), Some(last)) = (row.get::<Option<uuid::Uuid>, _>("scope_id"), flags.last_mut()) {
                last.2.push((
                    depths.get(&id).copied().unwrap_or(i32::MAX),
                    ScopeId::from_uuid(id),
                    row.get("enabled"),
                    row.get("locked"),
                ));
            }
        }
        Ok(flags
            .into_iter()
            .map(|(code, default, set)| {
                let (enabled, origin) = choose(set).unwrap_or((default, Origin::Declared));
                Flag {
                    code,
                    enabled,
                    origin,
                }
            })
            .collect())
    }
}
