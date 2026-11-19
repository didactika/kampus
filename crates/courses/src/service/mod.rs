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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The service API of courses, one file per area, and the checks every area shares.

pub mod category;
pub mod course;
pub mod field;
pub mod group;
pub mod reference;
pub mod section;
pub mod visibility;

use kampus_common::{Error, ErrorKind, FieldError, Result};

/// The longest name or title, in characters.
const MAX_NAME: usize = 255;

/// A taken unique value answered as a conflict with this module's code.
pub(crate) fn taken(code: &'static str) -> impl Fn(Error) -> Error {
    move |error| {
        if error.code() == "db.unique_violation" {
            Error::conflict(code)
        } else {
            error
        }
    }
}

/// Refuses a change made on another version than the stored one.
pub(crate) fn check_version(expected: Option<i32>, stored: i32, code: &'static str) -> Result<()> {
    if expected.is_some_and(|v| v != stored) {
        return Err(Error::new(ErrorKind::PreconditionFailed, code).with_param("version", stored.to_string()));
    }
    Ok(())
}

/// A required name: trimmed, not empty, not too long.
pub(crate) fn required(field: &'static str, value: &str, problems: &mut Vec<FieldError>) -> String {
    let value = value.trim();
    if value.is_empty() {
        problems.push(FieldError::new(field, "required"));
    } else if value.chars().count() > MAX_NAME {
        problems.push(FieldError::new(field, "length"));
    }
    value.to_owned()
}

/// An optional text: trimmed, empty as absent.
pub(crate) fn optional(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// The answer to the problems found, every one at once.
pub(crate) fn valid(code: &'static str, problems: Vec<FieldError>) -> Result<()> {
    if problems.is_empty() {
        Ok(())
    } else {
        Err(Error::validation(code, problems))
    }
}

/// An instant of a row.
pub(crate) fn instant(row: &sqlx::postgres::PgRow, column: &str) -> Option<kampus_common::Timestamp> {
    use sqlx::Row;
    row.get::<Option<time::OffsetDateTime>, _>(column)
        .map(kampus_common::Timestamp::from_offset_date_time)
}

/// The own visibility of a row.
pub(crate) fn visibility_of(row: &sqlx::postgres::PgRow) -> crate::Visibility {
    use sqlx::Row;
    crate::Visibility::read(
        row.get::<String, _>("visibility").as_str(),
        instant(row, "visible_from"),
        instant(row, "visible_until"),
    )
}

/// The window of a visibility, as bound.
pub(crate) fn window_of(
    visibility: crate::Visibility,
) -> (Option<time::OffsetDateTime>, Option<time::OffsetDateTime>) {
    let (from, until) = visibility.window();
    (
        from.map(|f| f.as_offset_date_time()),
        until.map(|u| u.as_offset_date_time()),
    )
}
