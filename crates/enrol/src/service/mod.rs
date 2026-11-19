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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The service API of enrolment, one file per area, and what the areas share.

pub mod enrolment;
pub mod expiry;
pub mod instance;
pub mod participation;
pub mod request;
pub mod self_enrol;
pub mod sync;

use kampus_common::{Error, ErrorKind, Result};

/// An instant of a row.
pub(crate) fn instant(row: &sqlx::postgres::PgRow, column: &str) -> Option<kampus_common::Timestamp> {
    use sqlx::Row;
    row.get::<Option<time::OffsetDateTime>, _>(column)
        .map(kampus_common::Timestamp::from_offset_date_time)
}

/// Refuses a change made on another version than the stored one.
pub(crate) fn check_version(expected: Option<i32>, stored: i32, code: &'static str) -> Result<()> {
    if expected.is_some_and(|v| v != stored) {
        return Err(Error::new(ErrorKind::PreconditionFailed, code).with_param("version", stored.to_string()));
    }
    Ok(())
}
