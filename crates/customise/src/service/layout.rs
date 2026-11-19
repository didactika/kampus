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

//! Layouts: the widgets of a page kind arranged in its regions per scope, validated against the regions
//! and the widgets the modules declare; a widget no module declares any more is skipped, never breaks the
//! page.

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use kampus_module::ValueKind;
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};

use super::{is_code, principal, refuse_below_lock};
use crate::permissions::LAYOUT_MANAGE;
use crate::{Customise, Layout, Origin, Placement, sql};

/// The most widgets a layout holds.
const MAX_PLACEMENTS: usize = 50;

/// A widget as declared: the page kinds it may be placed in and its settings.
struct Widget {
    /// The page kinds; every one when empty.
    page_kinds: Vec<String>,
    /// Its settings as stored: key, kind and default.
    settings: Vec<Value>,
}

impl Widget {
    /// The declared setting of a key.
    fn setting(&self, key: &str) -> Option<&Value> {
        self.settings
            .iter()
            .find(|s| s.get("key").and_then(Value::as_str) == Some(key))
    }

    /// The settings of a placement with the defaults of what it leaves out.
    fn with_defaults(&self, mut settings: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
        for declared in &self.settings {
            if let (Some(key), Some(default)) = (
                declared.get("key").and_then(Value::as_str),
                declared.get("default"),
            ) {
                settings.entry(key.to_owned()).or_insert_with(|| default.clone());
            }
        }
        settings
    }
}

/// Reads a widget from its row.
fn widget(row: &sqlx::postgres::PgRow, settings: &str) -> Widget {
    let strings = |v: Value| -> Vec<Value> { v.as_array().cloned().unwrap_or_default() };
    Widget {
        page_kinds: strings(row.get("page_kinds"))
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        settings: strings(row.get(settings)),
    }
}

/// The problems of the placements of a layout of a page kind.
fn problems(
    page_kind: &str,
    regions: &[String],
    widgets: &BTreeMap<String, Widget>,
    placements: &[Placement],
) -> Vec<FieldError> {
    let mut problems = Vec::new();
    if placements.len() > MAX_PLACEMENTS {
        problems.push(FieldError::new("placements", "length"));
    }
    for (at, placement) in placements.iter().enumerate() {
        let name = |part: &str| format!("placements[{at}].{part}");
        if !regions.contains(&placement.region) {
            problems.push(FieldError::new(name("region"), "one_of"));
        }
        let Some(widget) = widgets.get(&placement.widget) else {
            problems.push(FieldError::new(name("widget"), "one_of"));
            continue;
        };
        if !widget.page_kinds.is_empty() && !widget.page_kinds.iter().any(|p| p == page_kind) {
            problems.push(FieldError::new(name("widget"), "page_kind"));
        }
        for (key, value) in &placement.settings {
            let fits = widget
                .setting(key)
                .and_then(|s| s.get("kind"))
                .is_some_and(|kind| ValueKind::stored_accepts(kind, value));
            if !fits {
                problems.push(FieldError::new(name(&format!("settings.{key}")), "type"));
            }
        }
        if placement.permission.as_deref().is_some_and(|p| !is_code(p)) {
            problems.push(FieldError::new(name("permission"), "code"));
        }
    }
    problems
}

impl Customise {
    /// The layout of a page kind in a scope, with the widgets the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.page_kind_unknown`) for a page kind no module declares.
    pub async fn layout(&self, actor: &Actor, page_kind: &str, scope: ScopeId) -> Result<Layout> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let regions = self.regions(&mut conn, page_kind).await?;
        let Some((id, origin)) = self
            .nearest(&mut conn, ("layout", "page_kind_code"), page_kind, scope)
            .await?
        else {
            return Ok(Layout {
                page_kind: page_kind.to_owned(),
                regions,
                placements: Vec::new(),
                origin: Origin::Declared,
            });
        };
        let rows = sqlx::query(sql(format!(
            "select p.region, p.widget_code, p.settings, p.permission, w.page_kinds, w.settings as declared
             from {} p join {} w on w.code = p.widget_code
             where p.layout_id = $1 and w.retired_at is null order by p.position",
            self.table("placement"),
            self.table("widget")
        )))
        .bind(id)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        drop(conn);
        let mut placements = Vec::new();
        for row in &rows {
            let region: String = row.get("region");
            let permission: Option<String> = row.get("permission");
            if !regions.contains(&region) || !self.holds(actor, permission.as_deref(), scope).await? {
                continue;
            }
            let settings: BTreeMap<String, Value> =
                serde_json::from_value(row.get("settings")).unwrap_or_default();
            placements.push(Placement {
                region,
                widget: row.get("widget_code"),
                settings: widget(row, "declared").with_defaults(settings),
                permission,
            });
        }
        placements.sort_by_key(|p| regions.iter().position(|r| *r == p.region));
        Ok(Layout {
            page_kind: page_kind.to_owned(),
            regions,
            placements,
            origin,
        })
    }

    /// Replaces the layout of a page kind in a scope, locked or not; needs `customise.layout.manage` there.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown page kind, `permission_denied` without the permission, `validation`
    /// (`customise.layout_invalid`) for an unknown region or widget, a widget not offered for the page kind
    /// or a setting it does not declare, `conflict` (`customise.locked_above`) below a scope that locks it.
    pub async fn set_layout(
        &self,
        actor: &Actor,
        page_kind: &str,
        scope: ScopeId,
        placements: Vec<Placement>,
        locked: bool,
    ) -> Result<Layout> {
        self.access.require(actor, LAYOUT_MANAGE, scope).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let regions = self.regions(&mut conn, page_kind).await?;
        let codes: Vec<&str> = placements.iter().map(|p| p.widget.as_str()).collect();
        let rows = sqlx::query(sql(format!(
            "select code, page_kinds, settings from {} where code = any($1) and retired_at is null",
            self.table("widget")
        )))
        .bind(&codes)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let widgets: BTreeMap<String, Widget> = rows
            .iter()
            .map(|r| (r.get("code"), widget(r, "settings")))
            .collect();
        let problems = problems(page_kind, &regions, &widgets, &placements);
        if !problems.is_empty() {
            return Err(Error::validation("customise.layout_invalid", problems));
        }
        if let Some((_, origin)) = self
            .nearest(&mut conn, ("layout", "page_kind_code"), page_kind, scope)
            .await?
        {
            refuse_below_lock(origin, scope)?;
        }
        drop(conn);
        let mut tx = self.db.begin().await?;
        self.write_layout_in(&mut tx, page_kind, scope, &placements, locked)
            .await?;
        self.record_in(
            &mut tx,
            "layout",
            page_kind,
            scope,
            principal(actor),
            (None, Some(json!({ "placements": placements, "locked": locked }))),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.layout(actor, page_kind, scope).await
    }

    /// Removes the layout a scope set for a page kind, so it comes from above again.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `customise.layout.manage`.
    pub async fn reset_layout(&self, actor: &Actor, page_kind: &str, scope: ScopeId) -> Result<Layout> {
        self.access.require(actor, LAYOUT_MANAGE, scope).await?;
        self.remove_in_scope(actor, ("layout", "page_kind_code"), page_kind, scope)
            .await?;
        self.layout(actor, page_kind, scope).await
    }

    /// The regions of a page kind, in order.
    async fn regions(&self, conn: &mut PgConnection, page_kind: &str) -> Result<Vec<String>> {
        let regions: Value = sqlx::query_scalar(sql(format!(
            "select regions from {} where code = $1 and retired_at is null",
            self.table("page_kind")
        )))
        .bind(page_kind)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| {
            Error::not_found("customise.page_kind_unknown").with_param("code", page_kind.to_owned())
        })?;
        Ok(serde_json::from_value(regions).unwrap_or_default())
    }

    /// Stores the layout of a page kind in a scope with its placements, replacing the ones it had.
    async fn write_layout_in(
        &self,
        conn: &mut PgConnection,
        page_kind: &str,
        scope: ScopeId,
        placements: &[Placement],
        locked: bool,
    ) -> Result<()> {
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {t} (id, page_kind_code, scope_id, locked) values ($1, $2, $3, $4)
             on conflict (page_kind_code, scope_id) do update set locked = excluded.locked,
             version = {t}.version + 1 returning id",
            t = self.table("layout")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(page_kind)
        .bind(scope.as_uuid())
        .bind(locked)
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "delete from {} where layout_id = $1",
            self.table("placement")
        )))
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for (position, placement) in (0_i32..).zip(placements) {
            sqlx::query(sql(format!(
                "insert into {} (id, layout_id, region, widget_code, settings, permission, position)
                 values ($1, $2, $3, $4, $5, $6, $7)",
                self.table("placement")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(id)
            .bind(&placement.region)
            .bind(&placement.widget)
            .bind(json!(placement.settings))
            .bind(placement.permission.as_deref())
            .bind(position)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    }
}
