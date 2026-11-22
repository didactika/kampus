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

//! The service API of customisation, one file per kind, and what the kinds share: resolution from the
//! nearest scope up with locks, and the record and event of every change.

pub mod declarations;
pub mod export;
pub mod field;
pub mod flag;
pub mod import;
pub mod layout;
pub mod menu;
pub mod rule;
pub mod run;
pub mod term;
pub mod theme;

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};

use crate::{Customise, Origin, sql};

/// Refuses to set in a scope what a scope above it locks.
pub(crate) fn refuse_below_lock(current: Origin, scope: ScopeId) -> Result<()> {
    match current {
        Origin::Scope {
            scope: at,
            locked: true,
        } if at != scope => Err(Error::conflict("customise.locked_above")),
        _ => Ok(()),
    }
}

/// Whether a text is a code: lowercase letters, digits, `_` and `.`.
pub(crate) fn is_code(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.')
}

/// A value set in a scope of a path: how far up it is, the scope, the value and whether it locks.
pub(crate) type SetIn<T> = (i32, ScopeId, T, bool);

/// The value that applies among those set along a path: the highest one that locks, or else the nearest.
pub(crate) fn choose<T>(mut set: Vec<SetIn<T>>) -> Option<(T, Origin)> {
    set.sort_by_key(|s| std::cmp::Reverse(s.0));
    let at = set
        .iter()
        .position(|(_, _, _, locked)| *locked)
        .or_else(|| set.len().checked_sub(1))?;
    let (_, scope, value, locked) = set.swap_remove(at);
    Some((value, Origin::Scope { scope, locked }))
}

/// The principal of an actor, for the record of a change.
pub(crate) const fn principal(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(p.as_uuid()),
        Actor::Visitor(_) => None,
    }
}

impl Customise {
    /// The scopes of a path from a scope to the root, with how far up each is.
    pub(crate) async fn depths(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
    ) -> Result<BTreeMap<uuid::Uuid, i32>> {
        Ok(self
            .access
            .path(conn, scope)
            .await?
            .iter()
            .map(|s| (s.id.as_uuid(), s.depth))
            .collect())
    }

    /// The row of a table, keyed by a column and a scope, that applies in a scope: the one of the highest
    /// scope that locks it, or else of the nearest; with where it came from.
    pub(crate) async fn nearest(
        &self,
        conn: &mut PgConnection,
        (table, column): (&str, &str),
        key: &str,
        scope: ScopeId,
    ) -> Result<Option<(uuid::Uuid, Origin)>> {
        let depths = self.depths(conn, scope).await?;
        let ids: Vec<uuid::Uuid> = depths.keys().copied().collect();
        let rows = sqlx::query(sql(format!(
            "select id, scope_id, locked from {} where {column} = $1 and scope_id = any($2)",
            self.table(table)
        )))
        .bind(key)
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let set: Vec<SetIn<uuid::Uuid>> = rows
            .iter()
            .map(|r| {
                let at: uuid::Uuid = r.get("scope_id");
                (
                    depths.get(&at).copied().unwrap_or(i32::MAX),
                    ScopeId::from_uuid(at),
                    r.get("id"),
                    r.get("locked"),
                )
            })
            .collect();
        Ok(choose(set))
    }

    /// Removes the row a scope set in a table keyed by a column, and records it when there was one.
    pub(crate) async fn remove_in_scope(
        &self,
        actor: &Actor,
        (table, column): (&str, &str),
        key: &str,
        scope: ScopeId,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where {column} = $1 and scope_id = $2",
            self.table(table)
        )))
        .bind(key)
        .bind(scope.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed > 0 {
            self.record_in(&mut tx, table, key, scope, principal(actor), (None, None))
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Whether the actor may change what a scope sets of a kind whose resolved value came from `origin`: they
    /// hold the permission there and no scope above locks it.
    pub(crate) async fn editable(
        &self,
        actor: &Actor,
        permission: &str,
        origin: &crate::Origin,
        scope: ScopeId,
    ) -> Result<bool> {
        let locked_above = matches!(origin, crate::Origin::Scope { scope: at, locked: true } if *at != scope);
        Ok(!locked_above && self.holds(actor, Some(permission), scope).await?)
    }

    /// Whether an actor holds a permission in a scope; everyone when no permission is given.
    pub(crate) async fn holds(
        &self,
        actor: &Actor,
        permission: Option<&str>,
        scope: ScopeId,
    ) -> Result<bool> {
        match permission {
            Some(permission) => Ok(self.access.check(actor, permission, scope).await?.allowed),
            None => Ok(true),
        }
    }

    /// Records a change of a customisation and publishes it, in the transaction of the change.
    pub(crate) async fn record_in(
        &self,
        conn: &mut PgConnection,
        kind: &str,
        object: &str,
        scope: ScopeId,
        actor: Option<uuid::Uuid>,
        (before, after): (Option<Value>, Option<Value>),
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, kind, object, scope_id, actor, before, after) values ($1, $2, $3, $4, $5, $6, $7)",
            self.table("change")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(kind)
        .bind(object)
        .bind(scope.as_uuid())
        .bind(actor)
        .bind(before)
        .bind(after)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut event = kampus_events::Publish::new(
            crate::events::CHANGED,
            Some(scope.as_uuid()),
            json!({ "kind": kind, "object": object, "scope": scope }),
        );
        event.scope = Some(scope.as_uuid());
        event.actor = actor;
        event.summary = Some(format!("{kind} {object} customised"));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }
}
