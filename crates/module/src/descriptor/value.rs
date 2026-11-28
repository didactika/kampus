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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The kinds of value a module declares for its preferences and settings: each one with its default,
//! and what it accepts.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A kind of value as it is stored and as a client reads it to build a field: its type and bounds, without
/// its default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoredKind {
    /// On or off.
    Boolean {},
    /// One of a list of options.
    Choice {
        /// The options.
        options: Vec<String>,
    },
    /// A whole number in a range.
    Integer {
        /// The smallest value.
        min: i64,
        /// The largest value.
        max: i64,
    },
    /// Text of at most some characters.
    Text {
        /// The longest text.
        max: usize,
    },
    /// Rich text in the one format of user content: at most some characters of HTML.
    RichText {
        /// The longest HTML.
        max: usize,
    },
}

impl From<ValueKind> for StoredKind {
    fn from(kind: ValueKind) -> Self {
        match kind {
            ValueKind::Boolean { .. } => Self::Boolean {},
            ValueKind::Choice { options, .. } => Self::Choice {
                options: options.iter().map(|&o| o.to_owned()).collect(),
            },
            ValueKind::Integer { min, max, .. } => Self::Integer { min, max },
            ValueKind::Text { max, .. } => Self::Text { max },
            ValueKind::RichText { max } => Self::RichText { max },
        }
    }
}

/// The type of a declared value, with its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// On or off.
    Boolean {
        /// The value when none was chosen.
        default: bool,
    },
    /// One of a list of options.
    Choice {
        /// The options.
        options: &'static [&'static str],
        /// The value when none was chosen.
        default: &'static str,
    },
    /// A whole number in a range.
    Integer {
        /// The smallest value.
        min: i64,
        /// The largest value.
        max: i64,
        /// The value when none was chosen.
        default: i64,
    },
    /// Text of at most some characters.
    Text {
        /// The longest text.
        max: usize,
        /// The value when none was chosen.
        default: &'static str,
    },
    /// Rich text in the one format of user content, kept sanitised: at most some characters of HTML, empty when none
    /// was chosen.
    RichText {
        /// The longest HTML.
        max: usize,
    },
}

impl ValueKind {
    /// The default, as a value.
    #[must_use]
    pub fn default_value(self) -> Value {
        match self {
            Self::Boolean { default } => Value::Bool(default),
            Self::Choice { default, .. } | Self::Text { default, .. } => Value::String(default.to_owned()),
            Self::RichText { .. } => Value::String(String::new()),
            Self::Integer { default, .. } => Value::from(default),
        }
    }

    /// Whether a value is of this kind and within its bounds.
    #[must_use]
    pub fn accepts(self, value: &Value) -> bool {
        match (self, value) {
            (Self::Boolean { .. }, Value::Bool(_)) => true,
            (Self::Choice { options, .. }, Value::String(text)) => options.contains(&text.as_str()),
            (Self::Integer { min, max, .. }, Value::Number(n)) => {
                n.as_i64().is_some_and(|n| (min..=max).contains(&n))
            }
            (Self::Text { max, .. } | Self::RichText { max }, Value::String(text)) => {
                text.chars().count() <= max
            }
            _ => false,
        }
    }

    /// The kind as stored, so a value is checked against it without the declaration in hand.
    #[must_use]
    pub fn stored(self) -> Value {
        serde_json::to_value(StoredKind::from(self)).unwrap_or(Value::Null)
    }

    /// Whether a value is of a kind as [`ValueKind::stored`] wrote it, and within its bounds.
    #[must_use]
    pub fn stored_accepts(kind: &Value, value: &Value) -> bool {
        let bound = |name: &str| kind.get(name).and_then(Value::as_i64);
        match (kind.get("type").and_then(Value::as_str), value) {
            (Some("boolean"), Value::Bool(_)) => true,
            (Some("integer"), Value::Number(n)) => match (n.as_i64(), bound("min"), bound("max")) {
                (Some(n), Some(min), Some(max)) => (min..=max).contains(&n),
                _ => false,
            },
            (Some("text" | "rich_text"), Value::String(text)) => bound("max")
                .and_then(|m| usize::try_from(m).ok())
                .is_some_and(|m| text.chars().count() <= m),
            (Some("choice"), Value::String(text)) => kind
                .get("options")
                .and_then(Value::as_array)
                .is_some_and(|o| o.iter().any(|x| x.as_str() == Some(text))),
            _ => false,
        }
    }
}
