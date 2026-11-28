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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The settings of a scope as its administration shows them: every setting that may be set there and that
//! the request may set, with its type, its value, where the value comes from and its locks.

use std::collections::BTreeMap;

use kampus_access::{Acting, ScopeId};
use kampus_common::{Error, Result};
use kampus_module::StoredKind;
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;

use crate::{Origin, Settings, sql};

/// A setting in a scope, as the administration of the scope shows and changes it.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct ScopeSetting {
    /// The key.
    pub key: String,
    /// The module that declares it.
    pub module: String,
    /// What it sets.
    pub summary: String,
    /// The group it shows in among the settings of its module.
    pub group: String,
    /// Its type and bounds, to build its field.
    pub kind: StoredKind,
    /// Its declared default.
    pub default_value: Value,
    /// Its value in the scope.
    pub value: Value,
    /// Where the value comes from.
    pub origin: Origin,
    /// The version of the value the scope itself sets, `0` when it sets none, so it takes the one from above:
    /// what `If-Match` carries to change it.
    pub version: i32,
    /// Whether the scope locks it for the scopes below.
    pub locked: bool,
    /// Whether a scope above locks it, so it cannot change here.
    pub locked_above: bool,
    /// The permission that sets it.
    pub permission: String,
    /// Whether a change applies only when the processes that read it start again.
    pub restart_required: bool,
}

impl Settings {
    /// The settings that may be set in a scope and that the request may set there, by key, filtered by
    /// words found in their key, module or summary.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn of_scope(
        &self,
        acting: &Acting,
        scope: ScopeId,
        words: Option<&str>,
    ) -> Result<Vec<ScopeSetting>> {
        self.of_scope_where(acting, scope, |s| {
            words.is_none_or(|w| {
                let w = w.trim().to_lowercase();
                [&s.0, &s.1, &s.2].iter().any(|t| t.to_lowercase().contains(&w))
            })
        })
        .await
    }

    /// One setting of a scope, as [`Settings::of_scope`] shows it.
    ///
    /// # Errors
    ///
    /// `not_found` (`settings.unknown`) for a key that is not declared, may not be set in the scope or the
    /// request may not set there.
    pub async fn in_scope(&self, acting: &Acting, key: &str, scope: ScopeId) -> Result<ScopeSetting> {
        self.of_scope_where(acting, scope, |s| s.0 == key)
            .await?
            .pop()
            .ok_or_else(|| Error::not_found("settings.unknown").with_param("key", key.to_owned()))
    }

    /// The settings of a scope whose key, module and summary a filter keeps.
    async fn of_scope_where(
        &self,
        acting: &Acting,
        scope: ScopeId,
        keep: impl Fn(&(String, String, String)) -> bool,
    ) -> Result<Vec<ScopeSetting>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let kind = self.access.path(&mut conn, scope).await?[0]
            .kind
            .as_str()
            .to_owned();
        let rows = sqlx::query(sql(format!(
            "select d.key, d.module, d.summary, d.group_code, d.kind, d.default_value, d.permission, d.restart_required,
                    v.version, v.locked
             from {} d left join {} v on v.definition_key = d.key and v.scope_id = $1
             where d.retired_at is null and d.levels ? $2 order by d.module, d.position, d.key",
            self.table("definition"),
            self.table("value")
        )))
        .bind(scope.as_uuid())
        .bind(&kind)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let resolved = self.resolve_prefix_in(&mut conn, "", scope).await?;
        drop(conn);
        let mut may: BTreeMap<String, bool> = BTreeMap::new();
        let mut shown = Vec::new();
        for row in &rows {
            let named: (String, String, String) = (row.get("key"), row.get("module"), row.get("summary"));
            if !keep(&named) {
                continue;
            }
            let permission: String = row.get("permission");
            if !may.contains_key(&permission) {
                let allowed = self.access.carries(acting, &permission, scope).await?
                    && self
                        .access
                        .check(&acting.actor, &permission, scope)
                        .await?
                        .allowed;
                may.insert(permission.clone(), allowed);
            }
            let (Some(true), Some(found), Ok(stored)) = (
                may.get(&permission).copied(),
                resolved.get(&named.0),
                serde_json::from_value::<StoredKind>(row.get("kind")),
            ) else {
                continue;
            };
            let version: Option<i32> = row.get("version");
            let locked_above =
                matches!(found.origin, Origin::Scope { scope: from, locked: true } if from != scope);
            shown.push(ScopeSetting {
                key: named.0,
                module: named.1,
                summary: named.2,
                group: row.get("group_code"),
                kind: stored,
                default_value: row.get("default_value"),
                value: found.value.clone(),
                origin: found.origin,
                version: version.unwrap_or(0),
                locked: row.get::<Option<bool>, _>("locked").unwrap_or(false),
                locked_above,
                permission,
                restart_required: row.get("restart_required"),
            });
        }
        Ok(shown)
    }
}
