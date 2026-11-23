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

//! The move of the widgets of the core into plugins of the widget type: each takes the name of its
//! plugin as its code, and the placements of existing layouts follow it.

use kampus_db::{ActionContext, ActionFuture};
use sqlx::{AssertSqlSafe, PgConnection};

/// The code each widget of the core had, and the name of its plugin.
pub const RENAMED: &[(&str, &str)] = &[
    ("customise.welcome", "widget_welcome"),
    ("customise.text", "widget_text"),
    ("courses.catalogue", "widget_catalogue"),
    ("notify.inbox", "widget_notifications"),
    ("courses.outline", "widget_course_content"),
    ("courses.summary", "widget_course_summary"),
    ("progress.course", "widget_course_progress"),
    ("records.grades", "widget_course_grades"),
    ("calendar.upcoming", "widget_upcoming"),
];

/// Gives every placement of a widget of the core the code of its plugin. The widget is declared under its new code
/// first, as it was, so the placements keep a widget to point to until the declarations are read again.
pub fn rename_widgets<'c>(conn: &'c mut PgConnection, context: &'c ActionContext) -> ActionFuture<'c> {
    Box::pin(async move {
        let (widget, placement) = (context.table("widget"), context.table("placement"));
        for (old, new) in RENAMED {
            sqlx::query(AssertSqlSafe(format!(
                "insert into {widget} (code, module, summary, page_kinds, sizes, settings, retired_at)
                 select $2, $2, summary, page_kinds, sizes, settings, retired_at from {widget} where code = $1
                 on conflict (code) do nothing"
            )))
            .bind(old)
            .bind(new)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
            sqlx::query(AssertSqlSafe(format!(
                "update {placement} set widget_code = $2 where widget_code = $1"
            )))
            .bind(old)
            .bind(new)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    })
}
