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

//! Layouts: the widgets of a page kind placed on its grid per scope, validated against the grid and the sizes the
//! widgets declare for it; a widget no module declares any more, or no longer for the page kind, is skipped and
//! never breaks the page.

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId, ScopeKind};
use kampus_common::{Error, FieldError, Result};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};

use super::gravity::{room, settle};
use super::values;
use super::{is_code, principal, refuse_below_lock};
use crate::permissions::{LAYOUT_MANAGE, LAYOUT_OWN};
use crate::{AvailableWidget, Customise, Layout, Origin, PageGrid, Placement, RowHeight, WidgetSizes, sql};

/// The most widgets a layout holds.
const MAX_PLACEMENTS: usize = 50;

/// The lowest row a widget may start on.
const MAX_ROW: u16 = 400;

/// A widget as declared: the sizes it takes on each page kind, whether a page holds it once at most, and its
/// settings.
pub(super) struct Widget {
    /// Whether a page holds it once at most.
    single: bool,
    /// Its sizes, by page kind; it is placed on no other.
    sizes: BTreeMap<String, WidgetSizes>,
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

    /// The settings of a placement with the defaults of what it leaves out, and without those the widget no longer
    /// declares, so a widget that drops a setting never breaks a page or its next save.
    fn with_defaults(&self, mut settings: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
        settings.retain(|key, _| self.setting(key).is_some());
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
pub(super) fn widget(row: &sqlx::postgres::PgRow, settings: &str) -> Widget {
    let sizes: Option<Value> = row.get("sizes");
    Widget {
        single: row.try_get::<bool, _>("single").unwrap_or(false),
        sizes: sizes
            .and_then(|s| serde_json::from_value(s).ok())
            .unwrap_or_default(),
        settings: row
            .get::<Value, _>(settings)
            .as_array()
            .cloned()
            .unwrap_or_default(),
    }
}

/// The problems of the placements of a layout of a page kind on its grid, which must hold the widgets every page of
/// the kind holds.
pub(super) fn problems(
    page_kind: &str,
    grid: PageGrid,
    (widgets, required): (&BTreeMap<String, Widget>, &[AvailableWidget]),
    placements: &[Placement],
) -> Vec<FieldError> {
    let mut problems = Vec::new();
    for kept in required {
        if !placements.iter().any(|p| p.widget == kept.code) {
            problems.push(FieldError::new(format!("placements.{}", kept.code), "required"));
        }
    }
    if placements.len() > MAX_PLACEMENTS {
        problems.push(FieldError::new("placements", "length"));
    }
    for (at, placement) in placements.iter().enumerate() {
        let name = |part: &str| format!("placements[{at}].{part}");
        let Some(widget) = widgets.get(&placement.widget) else {
            problems.push(FieldError::new(name("widget"), "one_of"));
            continue;
        };
        let Some(sizes) = widget.sizes.get(page_kind) else {
            problems.push(FieldError::new(name("widget"), "page_kind"));
            continue;
        };
        if !(sizes.min.w..=sizes.max.w).contains(&placement.w) {
            problems.push(FieldError::new(name("w"), "range"));
        }
        let (low, high) = sizes.heights(placement.w);
        if !(low..=high).contains(&placement.h) {
            problems.push(FieldError::new(name("h"), "range"));
        }
        if u32::from(placement.x) + u32::from(placement.w) > u32::from(grid.columns) {
            problems.push(FieldError::new(name("x"), "range"));
        }
        if placement.y > MAX_ROW {
            problems.push(FieldError::new(name("y"), "range"));
        }
        if widget.single
            && placements[..at]
                .iter()
                .any(|other| other.widget == placement.widget)
        {
            problems.push(FieldError::new(name("widget"), "single"));
        }
        if placements[..at].iter().any(|other| other.overlaps(placement)) {
            problems.push(FieldError::new(name("y"), "overlap"));
        }
        for (key, value) in &placement.settings {
            let rule = widget
                .setting(key)
                .map_or(Some("type"), |declared| values::problem(declared, value));
            if let Some(rule) = rule {
                problems.push(FieldError::new(name(&format!("settings.{key}")), rule));
            }
        }
        // A setting that may not be left empty is refused when the placement leaves it out, too.
        for declared in &widget.settings {
            let Some(key) = declared.get("key").and_then(Value::as_str) else {
                continue;
            };
            if !placement.settings.contains_key(key) {
                let default = declared.get("default").cloned().unwrap_or(Value::Null);
                if values::problem(declared, &default) == Some("required") {
                    problems.push(FieldError::new(name(&format!("settings.{key}")), "required"));
                }
            }
        }
        if placement.permission.as_deref().is_some_and(|p| !is_code(p)) {
            problems.push(FieldError::new(name("permission"), "code"));
        }
    }
    problems
}

/// Puts on a page the widgets the scopes above lock on it, as they placed them and pinned there: each replaces what
/// the page has of the same widget when a page holds it once, or the same widget at the same place.
fn hold(placements: &mut Vec<(Placement, bool)>, held: Vec<(Placement, bool)>) {
    for (locked, single) in held {
        placements
            .retain(|(p, _)| p.widget != locked.widget || (!single && (p.x, p.y) != (locked.x, locked.y)));
        if placements.iter().any(|(p, _)| {
            p.held && p.widget == locked.widget && (single || (p.x, p.y) == (locked.x, locked.y))
        }) {
            continue;
        }
        placements.push((
            Placement {
                pinned: true,
                held: true,
                ..locked
            },
            single,
        ));
    }
}

/// The grid of a page kind, as stored.
fn grid_of(stored: Option<Value>) -> PageGrid {
    let columns = stored
        .as_ref()
        .and_then(|g| g.get("columns"))
        .and_then(Value::as_u64)
        .and_then(|c| u16::try_from(c).ok())
        .unwrap_or(12);
    let row = stored
        .and_then(|g| g.get("row").cloned())
        .and_then(|r| serde_json::from_value(r).ok())
        .unwrap_or(RowHeight::Regular);
    let row_pixels = match row {
        RowHeight::Compact => 48,
        RowHeight::Regular => 72,
        RowHeight::Tall => 96,
    };
    PageGrid {
        columns,
        row,
        row_pixels,
    }
}

impl Customise {
    /// Whether an actor may arrange a kind of page in a scope: they hold `customise.layout.manage` there, or the
    /// page is personal, the scope is their own and they hold `customise.layout.own`.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn arranges(&self, actor: &Actor, page_kind: &str, scope: ScopeId) -> Result<bool> {
        if self.access.check(actor, LAYOUT_MANAGE, scope).await?.allowed {
            return Ok(true);
        }
        let Actor::Principal(principal) = *actor else {
            return Ok(false);
        };
        if !self.personal(page_kind) || self.owners.owner_of(scope).await? != Some(principal) {
            return Ok(false);
        }
        Ok(self.access.check(actor, LAYOUT_OWN, scope).await?.allowed)
    }

    /// Fails with `permission_denied` unless the actor may arrange the page in the scope.
    async fn require_arranges(&self, actor: &Actor, page_kind: &str, scope: ScopeId) -> Result<()> {
        if self.arranges(actor, page_kind, scope).await? {
            Ok(())
        } else {
            Err(Error::permission_denied("access.denied")
                .with_param("permission", LAYOUT_MANAGE)
                .with_param("rule", "no_allow"))
        }
    }

    /// The layout of a page kind in a scope, with the widgets the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.page_kind_unknown`) for a page kind no module declares.
    pub async fn layout(&self, actor: &Actor, page_kind: &str, scope: ScopeId) -> Result<Layout> {
        // Everything is read on one connection, which is given back before any permission is checked: a check takes
        // a connection of its own, and holding one while waiting for another starves the pool under load.
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let grid = self.grid(&mut conn, page_kind).await?;
        let nearest = self
            .nearest(&mut conn, ("layout", "page_kind_code"), page_kind, scope)
            .await?;
        let available = self.available(&mut conn, page_kind).await?;
        let rows = match &nearest {
            Some((id, _)) => sqlx::query(sql(format!(
                "select p.widget_code, p.x, p.y, p.w, p.h, p.pinned, p.locked, p.settings, p.permission, w.single, w.sizes, w.settings as declared
                 from {} p join {} w on w.code = p.widget_code
                 where p.layout_id = $1 and w.retired_at is null order by p.y, p.x",
                self.table("placement"),
                self.table("widget")
            )))
            .bind(*id)
            .fetch_all(&mut *conn)
            .await
            .map_err(kampus_db::classify)?,
            None => Vec::new(),
        };
        let origin = nearest.map_or(Origin::Declared, |(_, origin)| origin);
        // The widgets locked by the scopes above the one the layout comes from.
        let held_rows = match origin {
            Origin::Scope { scope: from, .. } => self.locked_above(&mut conn, page_kind, from).await?,
            Origin::Declared => Vec::new(),
        };
        // Only a scope that holds others arranging the same pages locks widgets for them: a platform, a tenant, a
        // campus or a category; a course, a section or a person has no such scopes below.
        let gathers = self
            .access
            .path(&mut conn, scope)
            .await?
            .iter()
            .find(|s| s.depth == 0)
            .is_some_and(|s| {
                matches!(
                    s.kind,
                    ScopeKind::Platform | ScopeKind::Tenant | ScopeKind::Campus | ScopeKind::Category
                )
            });
        drop(conn);
        let locked_above = matches!(origin, Origin::Scope { scope: at, locked: true } if at != scope);
        let editable = !locked_above && self.arranges(actor, page_kind, scope).await?;
        let lockable = editable && gathers;
        let held = self
            .read_placements(actor, page_kind, scope, grid, &held_rows)
            .await?;
        let mut placements = self.read_placements(actor, page_kind, scope, grid, &rows).await?;
        hold(&mut placements, held);
        // A widget every page holds is on the page even when the layout was written before it was declared so: it takes
        // the first room that holds it, and the next arrangement saved keeps it there.
        if nearest.is_some() {
            for kept in available.iter().filter(|w| w.required) {
                if !placements.iter().any(|(p, _)| p.widget == kept.code) {
                    let (w, h) = (kept.sizes.default.w.min(grid.columns), kept.sizes.default.h);
                    let taken: Vec<Placement> = placements.iter().map(|(p, _)| p.clone()).collect();
                    let (x, y) = room(&taken, w, h, grid.columns);
                    let settings = kept
                        .settings
                        .iter()
                        .filter_map(|s| Some((s.get("key")?.as_str()?.to_owned(), s.get("default")?.clone())))
                        .collect();
                    placements.push((
                        Placement {
                            widget: kept.code.clone(),
                            x,
                            y,
                            w,
                            h,
                            pinned: false,
                            locked: false,
                            held: false,
                            grows: kept.sizes.grows,
                            settings,
                            permission: None,
                        },
                        true,
                    ));
                }
            }
        }
        let placements = settle(placements.into_iter().map(|(p, _)| p).collect());
        let widgets = if editable { available } else { Vec::new() };
        Ok(Layout {
            page_kind: page_kind.to_owned(),
            grid,
            placements,
            origin,
            editable,
            lockable,
            widgets,
        })
    }

    /// The placements of rows as the actor sees them: the widgets it may see that the page kind still offers, each at
    /// the nearest size it now allows, a widget a page holds once only once, and their settings with their defaults.
    async fn read_placements(
        &self,
        actor: &Actor,
        page_kind: &str,
        scope: ScopeId,
        grid: PageGrid,
        rows: &[sqlx::postgres::PgRow],
    ) -> Result<Vec<(Placement, bool)>> {
        let mut placements: Vec<(Placement, bool)> = Vec::new();
        for row in rows {
            let declared = widget(row, "declared");
            let Some(sizes) = declared.sizes.get(page_kind) else {
                continue;
            };
            let code: String = row.get("widget_code");
            // A widget a page holds once shows once, in reading order, even when a layout written before it said so
            // holds more: the page never breaks, and the next arrangement saved keeps only the one shown.
            if declared.single && placements.iter().any(|(p, _)| p.widget == code) {
                continue;
            }
            let permission: Option<String> = row.get("permission");
            if !self.holds(actor, permission.as_deref(), scope).await? {
                continue;
            }
            let settings: BTreeMap<String, Value> =
                serde_json::from_value(row.get("settings")).unwrap_or_default();
            let cell = |column: &str| u16::try_from(row.get::<i32, _>(column)).unwrap_or(0);
            // A widget whose sizes changed since it was placed takes the nearest size it now allows.
            let w = cell("w").clamp(sizes.min.w, sizes.max.w).min(grid.columns);
            let (low, high) = sizes.heights(w);
            placements.push((
                Placement {
                    widget: code,
                    x: cell("x").min(grid.columns.saturating_sub(w)),
                    y: cell("y"),
                    w,
                    h: cell("h").clamp(low, high),
                    pinned: row.get("pinned"),
                    locked: row.get("locked"),
                    held: false,
                    grows: sizes.grows,
                    settings: values::clean(&declared.settings, declared.with_defaults(settings)),
                    permission,
                },
                declared.single,
            ));
        }
        Ok(placements)
    }

    /// The rows of the widgets the scopes above one lock on a page kind, the highest scope first.
    async fn locked_above(
        &self,
        conn: &mut PgConnection,
        page_kind: &str,
        scope: ScopeId,
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        let depths = self.depths(conn, scope).await?;
        let above: Vec<uuid::Uuid> = depths
            .iter()
            .filter(|(_, depth)| **depth > 0)
            .map(|(id, _)| *id)
            .collect();
        if above.is_empty() {
            return Ok(Vec::new());
        }
        let mut rows = sqlx::query(sql(format!(
            "select p.widget_code, p.x, p.y, p.w, p.h, p.pinned, p.locked, p.settings, p.permission, w.single, w.sizes,
                    w.settings as declared, l.scope_id
             from {} p join {} l on l.id = p.layout_id join {} w on w.code = p.widget_code
             where l.page_kind_code = $1 and l.scope_id = any($2) and p.locked and w.retired_at is null",
            self.table("placement"),
            self.table("layout"),
            self.table("widget")
        )))
        .bind(page_kind)
        .bind(&above)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        rows.sort_by_key(|r| {
            std::cmp::Reverse(
                depths
                    .get(&r.get::<uuid::Uuid, _>("scope_id"))
                    .copied()
                    .unwrap_or(0),
            )
        });
        Ok(rows)
    }

    /// The widgets that may be placed on a page kind, with their sizes there and their settings as declared.
    async fn available(&self, conn: &mut PgConnection, page_kind: &str) -> Result<Vec<AvailableWidget>> {
        let rows = sqlx::query(sql(format!(
            "select code, summary, icon, single, required, sizes, settings from {} where retired_at is null order by code",
            self.table("widget")
        )))
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let declared = widget(row, "settings");
                let sizes = declared.sizes.get(page_kind)?.clone();
                Some(AvailableWidget {
                    code: row.get("code"),
                    summary: row.get("summary"),
                    icon: row.get::<Option<String>, _>("icon").unwrap_or_default(),
                    single: declared.single,
                    required: row.get("required"),
                    sizes,
                    settings: declared.settings,
                })
            })
            .collect())
    }

    /// Replaces the layout of a page kind in a scope, locked or not.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown page kind, `permission_denied` when the actor may not arrange it, `validation`
    /// (`customise.layout_invalid`) for an unknown widget, one not offered for the page kind, a rectangle outside
    /// the grid or the sizes of its widget, two rectangles that overlap, a second of a widget a page holds once, or a
    /// setting the widget does not declare,
    /// `conflict` (`customise.locked_above`) below a scope that locks it.
    pub async fn set_layout(
        &self,
        actor: &Actor,
        page_kind: &str,
        scope: ScopeId,
        placements: Vec<Placement>,
        locked: bool,
    ) -> Result<Layout> {
        self.require_arranges(actor, page_kind, scope).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let grid = self.grid(&mut conn, page_kind).await?;
        // The widgets the scopes above lock here are theirs: what the writer sends of them is left out, and a widget
        // every page holds counts as held when they hold it.
        let held_rows = self.locked_above(&mut conn, page_kind, scope).await?;
        drop(conn);
        let held: Vec<String> = held_rows
            .iter()
            .map(|r| r.get::<String, _>("widget_code"))
            .collect();
        let held_once: Vec<String> = held_rows
            .iter()
            .filter(|r| r.get::<bool, _>("single"))
            .map(|r| r.get::<String, _>("widget_code"))
            .collect();
        let placements: Vec<Placement> = placements
            .into_iter()
            .filter(|p| !p.held && !held_once.contains(&p.widget))
            .map(|p| Placement { held: false, ..p })
            .collect();
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let widgets = self.widgets(&mut conn, &placements).await?;
        let required: Vec<AvailableWidget> = self
            .required(&mut conn, page_kind)
            .await?
            .into_iter()
            .filter(|w| !held.contains(&w.code))
            .collect();
        let problems = problems(page_kind, grid, (&widgets, &required), &placements);
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
        // What is stored is what the page shows: every widget not pinned risen to the ones above it, and rich text
        // sanitised.
        let placements: Vec<Placement> = settle(placements)
            .into_iter()
            .map(|p| {
                let settings = widgets.get(&p.widget).map_or_else(
                    || p.settings.clone(),
                    |w| values::clean(&w.settings, p.settings.clone()),
                );
                Placement { settings, ..p }
            })
            .collect();
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
    /// `permission_denied` when the actor may not arrange it.
    pub async fn reset_layout(&self, actor: &Actor, page_kind: &str, scope: ScopeId) -> Result<Layout> {
        self.require_arranges(actor, page_kind, scope).await?;
        self.remove_in_scope(actor, ("layout", "page_kind_code"), page_kind, scope)
            .await?;
        self.layout(actor, page_kind, scope).await
    }

    /// The widgets every page of a kind holds.
    pub(super) async fn required(
        &self,
        conn: &mut PgConnection,
        page_kind: &str,
    ) -> Result<Vec<AvailableWidget>> {
        Ok(self
            .available(conn, page_kind)
            .await?
            .into_iter()
            .filter(|w| w.required)
            .collect())
    }

    /// The declared widgets that placements name, by code.
    pub(super) async fn widgets(
        &self,
        conn: &mut PgConnection,
        placements: &[Placement],
    ) -> Result<BTreeMap<String, Widget>> {
        let codes: Vec<&str> = placements.iter().map(|p| p.widget.as_str()).collect();
        let rows = sqlx::query(sql(format!(
            "select code, single, sizes, settings from {} where code = any($1) and retired_at is null",
            self.table("widget")
        )))
        .bind(&codes)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (r.get("code"), widget(r, "settings")))
            .collect())
    }

    /// The grid of a page kind.
    pub(super) async fn grid(&self, conn: &mut PgConnection, page_kind: &str) -> Result<PageGrid> {
        let grid: Option<Value> = sqlx::query_scalar(sql(format!(
            "select grid from {} where code = $1 and retired_at is null",
            self.table("page_kind")
        )))
        .bind(page_kind)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| {
            Error::not_found("customise.page_kind_unknown").with_param("code", page_kind.to_owned())
        })?;
        Ok(grid_of(grid))
    }

    /// Stores the layout of a page kind in a scope with its placements, replacing the ones it had.
    pub(super) async fn write_layout_in(
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
        for placement in placements {
            sqlx::query(sql(format!(
                "insert into {} (id, layout_id, widget_code, x, y, w, h, pinned, locked, settings, permission)
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                self.table("placement")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(id)
            .bind(&placement.widget)
            .bind(i32::from(placement.x))
            .bind(i32::from(placement.y))
            .bind(i32::from(placement.w))
            .bind(i32::from(placement.h))
            .bind(placement.pinned)
            .bind(placement.locked)
            .bind(json!(placement.settings))
            .bind(placement.permission.as_deref())
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    }
}
