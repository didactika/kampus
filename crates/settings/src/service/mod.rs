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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The service API of settings, one file per area, and the stored form of a declared kind.

pub mod change;
pub mod declarations;
pub mod resolve;

use kampus_module::ValueKind;
use serde_json::{Value, json};

/// A declared kind as stored, so a value is checked against it without the declaration in hand.
pub(crate) fn kind_json(kind: ValueKind) -> Value {
    match kind {
        ValueKind::Boolean { .. } => json!({ "type": "boolean" }),
        ValueKind::Choice { options, .. } => json!({ "type": "choice", "options": options }),
        ValueKind::Integer { min, max, .. } => json!({ "type": "integer", "min": min, "max": max }),
        ValueKind::Text { max, .. } => json!({ "type": "text", "max": max }),
    }
}

/// Whether a value is of a stored kind and within its bounds.
pub(crate) fn accepts(kind: &Value, value: &Value) -> bool {
    let bound = |name: &str| kind.get(name).and_then(Value::as_i64);
    match (kind.get("type").and_then(Value::as_str), value) {
        (Some("boolean"), Value::Bool(_)) => true,
        (Some("integer"), Value::Number(n)) => match (n.as_i64(), bound("min"), bound("max")) {
            (Some(n), Some(min), Some(max)) => (min..=max).contains(&n),
            _ => false,
        },
        (Some("text"), Value::String(text)) => bound("max")
            .and_then(|m| usize::try_from(m).ok())
            .is_some_and(|m| text.chars().count() <= m),
        (Some("choice"), Value::String(text)) => kind
            .get("options")
            .and_then(Value::as_array)
            .is_some_and(|o| o.iter().any(|x| x.as_str() == Some(text))),
        _ => false,
    }
}
