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

//! Navigation menus: the items of a kind of menu per scope, as data, each shown only to the people who hold
//! its permission.

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde_json::json;
use sqlx::{PgConnection, Row};

use super::{is_code, principal, refuse_below_lock};
use crate::permissions::MENU_MANAGE;
use crate::{Customise, Menu, MenuItem, Origin, sql};

/// The most items a menu holds, at every depth.
const MAX_ITEMS: usize = 100;

/// The deepest an item may be, the top items being at depth one.
const MAX_DEPTH: usize = 3;

/// The longest label, in characters.
const MAX_LABEL: usize = 100;

/// The longest target, in characters.
const MAX_TARGET: usize = 2000;

/// An item ready to store: its id, its parent and its position among its siblings.
pub(super) type Flat<'a> = (uuid::Uuid, Option<uuid::Uuid>, i32, &'a MenuItem);

/// Whether a target is a path of the site or a web address.
fn is_target(target: &str) -> bool {
    let web = target.starts_with("https://") || target.starts_with("http://");
    let path = target.starts_with('/') && !target.starts_with("//");
    (web || path) && target.chars().count() <= MAX_TARGET && !target.chars().any(char::is_whitespace)
}

/// The items of a menu in the order they are stored, parents first, with the problems they have.
pub(super) fn flatten(items: &[MenuItem]) -> (Vec<Flat<'_>>, Vec<FieldError>) {
    let (mut flat, mut problems) = (Vec::new(), Vec::new());
    let mut pending: Vec<(Option<uuid::Uuid>, usize, String, &[MenuItem])> =
        vec![(None, 1, "items".to_owned(), items)];
    while let Some((parent, depth, path, siblings)) = pending.pop() {
        for (position, item) in (0_i32..).zip(siblings) {
            let name = format!("{path}[{position}]");
            let label = item.label.trim();
            if label.is_empty() || label.chars().count() > MAX_LABEL {
                problems.push(FieldError::new(format!("{name}.label"), "length"));
            }
            if !is_target(&item.target) {
                problems.push(FieldError::new(format!("{name}.target"), "target"));
            }
            if item.permission.as_deref().is_some_and(|p| !is_code(p)) {
                problems.push(FieldError::new(format!("{name}.permission"), "code"));
            }
            let id = uuid::Uuid::now_v7();
            flat.push((id, parent, position, item));
            if !item.children.is_empty() {
                if depth >= MAX_DEPTH {
                    problems.push(FieldError::new(format!("{name}.children"), "depth"));
                } else {
                    pending.push((Some(id), depth + 1, format!("{name}.children"), &item.children));
                }
            }
        }
    }
    if flat.len() > MAX_ITEMS {
        problems.push(FieldError::new("items", "length"));
    }
    (flat, problems)
}

/// A stored item: its parent, what it says, where it leads and who sees it.
pub(super) struct Stored {
    /// Its id.
    pub(super) id: uuid::Uuid,
    /// Its parent, if any.
    pub(super) parent: Option<uuid::Uuid>,
    /// The item, without its children.
    pub(super) item: MenuItem,
}

impl Customise {
    /// The menu of a kind in a scope, with the items the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.menu_kind_unknown`) for a kind of menu no module declares.
    pub async fn menu(&self, actor: &Actor, kind: &str, scope: ScopeId) -> Result<Menu> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.require_menu_kind(&mut conn, kind).await?;
        let Some((id, origin)) = self
            .nearest(&mut conn, ("menu", "menu_kind_code"), kind, scope)
            .await?
        else {
            drop(conn);
            return Ok(Menu {
                kind: kind.to_owned(),
                items: Vec::new(),
                origin: Origin::Declared,
                editable: self
                    .editable(actor, MENU_MANAGE, &Origin::Declared, scope)
                    .await?,
            });
        };
        let rows = sqlx::query(sql(format!(
            "select id, parent_id, label, target, permission from {} where menu_id = $1 order by position",
            self.table("menu_item")
        )))
        .bind(id)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        drop(conn);
        let mut seen: BTreeMap<String, bool> = BTreeMap::new();
        let mut stored = Vec::new();
        for row in &rows {
            let permission: Option<String> = row.get("permission");
            if let Some(code) = &permission
                && !seen.contains_key(code)
            {
                let held = self.holds(actor, Some(code), scope).await?;
                seen.insert(code.clone(), held);
            }
            if permission.as_ref().is_some_and(|p| !seen[p]) {
                continue;
            }
            stored.push(Stored {
                id: row.get("id"),
                parent: row.get("parent_id"),
                item: MenuItem {
                    label: row.get("label"),
                    target: row.get("target"),
                    permission,
                    children: Vec::new(),
                },
            });
        }
        let editable = self.editable(actor, MENU_MANAGE, &origin, scope).await?;
        Ok(Menu {
            kind: kind.to_owned(),
            items: tree(&stored, None),
            origin,
            editable,
        })
    }

    /// Replaces the menu of a kind in a scope, locked or not; needs `customise.menu.manage` there.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown kind of menu, `permission_denied` without the permission, `validation`
    /// (`customise.menu_invalid`) for empty or long labels, targets that are neither a path nor a web
    /// address, too many items or too deep, `conflict` (`customise.locked_above`) below a scope that locks it.
    pub async fn set_menu(
        &self,
        actor: &Actor,
        kind: &str,
        scope: ScopeId,
        items: Vec<MenuItem>,
        locked: bool,
    ) -> Result<Menu> {
        self.access.require(actor, MENU_MANAGE, scope).await?;
        let (flat, problems) = flatten(&items);
        if !problems.is_empty() {
            return Err(Error::validation("customise.menu_invalid", problems));
        }
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.require_menu_kind(&mut conn, kind).await?;
        if let Some((_, origin)) = self
            .nearest(&mut conn, ("menu", "menu_kind_code"), kind, scope)
            .await?
        {
            refuse_below_lock(origin, scope)?;
        }
        drop(conn);
        let mut tx = self.db.begin().await?;
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {t} (id, menu_kind_code, scope_id, locked) values ($1, $2, $3, $4)
             on conflict (menu_kind_code, scope_id) do update set locked = excluded.locked,
             version = {t}.version + 1 returning id",
            t = self.table("menu")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(kind)
        .bind(scope.as_uuid())
        .bind(locked)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.write_items_in(&mut tx, id, &flat).await?;
        self.record_in(
            &mut tx,
            "menu",
            kind,
            scope,
            principal(actor),
            (None, Some(json!({ "items": items, "locked": locked }))),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.menu(actor, kind, scope).await
    }

    /// Removes the menu a scope set for a kind, so it comes from above again.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `customise.menu.manage`.
    pub async fn reset_menu(&self, actor: &Actor, kind: &str, scope: ScopeId) -> Result<Menu> {
        self.access.require(actor, MENU_MANAGE, scope).await?;
        self.remove_in_scope(actor, ("menu", "menu_kind_code"), kind, scope)
            .await?;
        self.menu(actor, kind, scope).await
    }

    /// Fails unless a module declares the kind of menu.
    pub(super) async fn require_menu_kind(&self, conn: &mut PgConnection, kind: &str) -> Result<()> {
        sqlx::query_scalar::<_, String>(sql(format!(
            "select code from {} where code = $1 and retired_at is null",
            self.table("menu_kind")
        )))
        .bind(kind)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .map(drop)
        .ok_or_else(|| Error::not_found("customise.menu_kind_unknown").with_param("code", kind.to_owned()))
    }

    /// Stores the items of a menu, parents first, replacing the ones it had.
    pub(super) async fn write_items_in(
        &self,
        conn: &mut PgConnection,
        menu: uuid::Uuid,
        flat: &[Flat<'_>],
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "delete from {} where menu_id = $1",
            self.table("menu_item")
        )))
        .bind(menu)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for (id, parent, position, item) in flat {
            sqlx::query(sql(format!(
                "insert into {} (id, menu_id, parent_id, label, target, permission, position)
                 values ($1, $2, $3, $4, $5, $6, $7)",
                self.table("menu_item")
            )))
            .bind(id)
            .bind(menu)
            .bind(parent)
            .bind(item.label.trim())
            .bind(&item.target)
            .bind(item.permission.as_deref())
            .bind(position)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    }
}

/// The items under a parent, in order, each with the items under it; an item hidden from the reader hides
/// the items under it too.
pub(super) fn tree(stored: &[Stored], parent: Option<uuid::Uuid>) -> Vec<MenuItem> {
    stored
        .iter()
        .filter(|s| s.parent == parent)
        .map(|s| MenuItem {
            children: tree(stored, Some(s.id)),
            ..s.item.clone()
        })
        .collect()
}
