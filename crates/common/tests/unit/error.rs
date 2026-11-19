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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Tests of `kampus-common`: error.

use kampus_common::{Error, ErrorKind, FieldError};

#[test]
fn every_kind_maps_to_its_status() {
    let expected = [
        (ErrorKind::NotFound, 404),
        (ErrorKind::PermissionDenied, 403),
        (ErrorKind::Validation, 422),
        (ErrorKind::Conflict, 409),
        (ErrorKind::PreconditionFailed, 412),
        (ErrorKind::RateLimited, 429),
        (ErrorKind::Unavailable, 503),
        (ErrorKind::Internal, 500),
    ];
    for (kind, status) in expected {
        assert_eq!(kind.http_status(), status, "{kind}");
    }
}

#[test]
fn validation_keeps_every_field_and_displays_them() {
    let error = Error::validation(
        "common.page_invalid",
        vec![
            FieldError::new("limit", "range"),
            FieldError::new("after", "cursor"),
        ],
    )
    .with_param("max", "200");
    assert_eq!(error.kind(), ErrorKind::Validation);
    assert_eq!(error.fields().len(), 2);
    assert_eq!(
        error.to_string(),
        "validation: common.page_invalid max=200 [limit: range] [after: cursor]"
    );
}

#[test]
fn the_cause_is_kept_as_source() {
    let error = Error::internal("db.query").with_source(std::io::Error::other("disk"));
    assert_eq!(
        std::error::Error::source(&error)
            .map(ToString::to_string)
            .as_deref(),
        Some("disk")
    );
}

#[test]
fn kinds_serialise_in_snake_case() {
    assert_eq!(
        serde_json::to_string(&ErrorKind::PreconditionFailed).unwrap(),
        "\"precondition_failed\""
    );
}
