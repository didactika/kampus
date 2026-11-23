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

//! The upgrade of layouts from regions to a grid: each placement of the top region becomes a row across the page,
//! those of the main region the left eight columns and those of the side the right four, one under the other in
//! their order, each at the height its widget most often takes.

use std::collections::BTreeMap;

use kampus_db::{ActionContext, ActionFuture};
use sqlx::{AssertSqlSafe, PgConnection, Row};

/// The columns of the grid the layouts are placed on.
const COLUMNS: i32 = 12;

/// The columns the main region becomes; the side takes the rest.
const MAIN: i32 = 8;

/// The height, in rows, a widget takes when it is converted, by code; four rows for any other.
fn height(widget: &str) -> i32 {
    match widget {
        "customise.text" | "courses.summary" | "progress.course" => 3,
        "notify.inbox" => 5,
        "courses.catalogue" => 8,
        "courses.outline" => 10,
        _ => 4,
    }
}

/// Places the widgets of every layout on the grid.
pub fn place_on_grid<'c>(conn: &'c mut PgConnection, context: &'c ActionContext) -> ActionFuture<'c> {
    Box::pin(async move {
        let table = context.table("placement");
        let rows = sqlx::query(AssertSqlSafe(format!(
            "select id, layout_id, region, widget_code from {table} order by layout_id, position"
        )))
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut layouts: BTreeMap<uuid::Uuid, Vec<(uuid::Uuid, String, String)>> = BTreeMap::new();
        for row in &rows {
            layouts.entry(row.get("layout_id")).or_default().push((
                row.get("id"),
                row.get::<Option<String>, _>("region").unwrap_or_default(),
                row.get("widget_code"),
            ));
        }
        for placements in layouts.values() {
            for (id, x, y, w, h) in arrange(placements) {
                sqlx::query(AssertSqlSafe(format!(
                    "update {table} set x = $2, y = $3, w = $4, h = $5 where id = $1"
                )))
                .bind(id)
                .bind(x)
                .bind(y)
                .bind(w)
                .bind(h)
                .execute(&mut *conn)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        Ok(())
    })
}

/// The rectangles of the placements of one layout, given in their order with their region and widget: the id of
/// each, its column, row, width and height.
#[must_use]
pub fn arrange(placements: &[(uuid::Uuid, String, String)]) -> Vec<(uuid::Uuid, i32, i32, i32, i32)> {
    let mut rectangles = Vec::new();
    let mut top = 0;
    for (id, _, widget) in placements.iter().filter(|(_, region, _)| region == "top") {
        rectangles.push((*id, 0, top, COLUMNS, height(widget)));
        top += height(widget);
    }
    let (mut main, mut side) = (top, top);
    for (id, region, widget) in placements.iter().filter(|(_, region, _)| region != "top") {
        if region == "side" {
            rectangles.push((*id, MAIN, side, COLUMNS - MAIN, height(widget)));
            side += height(widget);
        } else {
            rectangles.push((*id, 0, main, MAIN, height(widget)));
            main += height(widget);
        }
    }
    rectangles
}
