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
pub mod field;
pub mod flag;
pub mod term;

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::Result;
use serde_json::{Value, json};
use sqlx::PgConnection;

use crate::{Customise, Origin, sql};

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
