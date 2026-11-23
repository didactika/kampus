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

//! Resolution: the value of a setting in a scope is the one of the nearest scope that sets it, unless a
//! scope above locks it; without any, its declared default. One query reads every scope of the path.

use std::collections::BTreeMap;

use kampus_access::ScopeId;
use kampus_common::{Error, Result};
use serde_json::Value;
use sqlx::{PgConnection, Row};

use crate::{Definition, Origin, Resolved, Settings, sql};

/// A value set in a scope of the path: how far up, the scope, the value, and whether it is locked.
type SetIn = (i32, ScopeId, Value, bool);

impl Settings {
    /// The value of a setting in a scope, with where it came from.
    ///
    /// # Errors
    ///
    /// `not_found` (`settings.unknown`) for a key no module declares, `not_found` for an unknown scope.
    pub async fn resolve(&self, key: &str, scope: ScopeId) -> Result<Resolved> {
        let mut found = self.resolve_where(scope, "d.key = $2", key).await?;
        found
            .remove(key)
            .ok_or_else(|| Error::not_found("settings.unknown").with_param("key", key.to_owned()))
    }

    /// Every setting whose key starts with a prefix, such as `identity.`, resolved in a scope, by key.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn resolve_prefix(&self, prefix: &str, scope: ScopeId) -> Result<BTreeMap<String, Resolved>> {
        self.resolve_where(scope, "starts_with(d.key, $2)", prefix).await
    }

    /// As [`Settings::resolve_prefix`], on a connection the caller holds, such as the transaction of a change.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn resolve_prefix_in(
        &self,
        conn: &mut PgConnection,
        prefix: &str,
        scope: ScopeId,
    ) -> Result<BTreeMap<String, Resolved>> {
        self.resolve_where_in(conn, scope, "starts_with(d.key, $2)", prefix)
            .await
    }

    /// The definitions of every declared setting, by key.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn definitions(&self) -> Result<Vec<Definition>> {
        let rows = sqlx::query(sql(format!(
            "select key, module, summary, default_value, levels, permission, restart_required from {}
             where retired_at is null order by key",
            self.table("definition")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Definition {
                key: r.get("key"),
                module: r.get("module"),
                summary: r.get("summary"),
                default_value: r.get("default_value"),
                levels: serde_json::from_value(r.get("levels")).unwrap_or_default(),
                permission: r.get("permission"),
                restart_required: r.get("restart_required"),
            })
            .collect())
    }

    /// Resolves the definitions a condition selects, in a scope.
    async fn resolve_where(
        &self,
        scope: ScopeId,
        condition: &str,
        argument: &str,
    ) -> Result<BTreeMap<String, Resolved>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.resolve_where_in(&mut conn, scope, condition, argument).await
    }

    /// Resolves the definitions a condition selects, in a scope, on a connection.
    async fn resolve_where_in(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        condition: &str,
        argument: &str,
    ) -> Result<BTreeMap<String, Resolved>> {
        let path = self.access.path(&mut *conn, scope).await?;
        let depth: BTreeMap<uuid::Uuid, i32> = path.iter().map(|s| (s.id.as_uuid(), s.depth)).collect();
        let ids: Vec<uuid::Uuid> = depth.keys().copied().collect();
        let rows = sqlx::query(sql(format!(
            "select d.key, d.default_value, v.scope_id, v.value, v.locked from {definitions} d
             left join {values} v on v.definition_key = d.key and v.scope_id = any($1)
             where d.retired_at is null and {condition}",
            definitions = self.table("definition"),
            values = self.table("value")
        )))
        .bind(&ids)
        .bind(argument)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut candidates: BTreeMap<String, (Value, Vec<SetIn>)> = BTreeMap::new();
        for row in &rows {
            let key: String = row.get("key");
            let entry = candidates
                .entry(key)
                .or_insert_with(|| (row.get::<Value, _>("default_value"), Vec::new()));
            if let Some(set_in) = row.get::<Option<uuid::Uuid>, _>("scope_id") {
                let level = depth.get(&set_in).copied().unwrap_or(i32::MAX);
                entry.1.push((
                    level,
                    ScopeId::from_uuid(set_in),
                    row.get("value"),
                    row.get("locked"),
                ));
            }
        }
        Ok(candidates
            .into_iter()
            .map(|(key, (default, mut set))| {
                set.sort_by_key(|s| std::cmp::Reverse(s.0));
                let chosen = set
                    .iter()
                    .find(|(_, _, _, locked)| *locked)
                    .or_else(|| set.last())
                    .map(|(_, scope, value, locked)| {
                        (
                            value.clone(),
                            Origin::Scope {
                                scope: *scope,
                                locked: *locked,
                            },
                        )
                    });
                let (value, origin) = chosen.unwrap_or((default, Origin::Default));
                (key.clone(), Resolved { key, value, origin })
            })
            .collect())
    }
}
