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

//! The values of the settings of a widget: a text that takes one value per language, a setting that may not be left
//! empty, and rich text, kept sanitised in the one format of user content whenever it is written or read.

use std::collections::BTreeMap;

use kampus_module::ValueKind;
use serde_json::Value;

/// Whether a key names a language: `en`, `es`, `pt-BR`.
pub(crate) fn is_language(key: &str) -> bool {
    let (code, region) = key.split_once('-').map_or((key, None), |(c, r)| (c, Some(r)));
    (2..=3).contains(&code.len())
        && code.chars().all(|c| c.is_ascii_lowercase())
        && region.is_none_or(|r| r.len() == 2 && r.chars().all(|c| c.is_ascii_uppercase()))
}

/// The texts of a value, one per language when it is translatable, or itself.
fn texts(value: &Value) -> Vec<&Value> {
    match value {
        Value::Object(by_language) => by_language.values().collect(),
        other => vec![other],
    }
}

/// What is wrong with the value of a declared setting; nothing when it fits.
#[must_use]
pub fn problem(declared: &Value, value: &Value) -> Option<&'static str> {
    let kind = declared.get("kind")?;
    let translatable = declared
        .get("translatable")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let required = declared.get("required").and_then(Value::as_bool).unwrap_or(false);
    if let Value::Object(by_language) = value {
        if !translatable {
            return Some("type");
        }
        if by_language.keys().any(|k| !is_language(k)) {
            return Some("language");
        }
    }
    if !texts(value)
        .iter()
        .all(|text| ValueKind::stored_accepts(kind, text))
    {
        return Some("type");
    }
    let empty = texts(value).iter().all(|text| {
        text.as_str()
            .is_some_and(|t| t.trim().is_empty() || t.trim() == "<p></p>")
    });
    if required && empty {
        return Some("required");
    }
    None
}

/// The settings with their rich text sanitised, in every language.
#[must_use]
pub fn clean(declared: &[Value], mut settings: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    for setting in declared {
        let rich = setting
            .get("kind")
            .and_then(|k| k.get("type"))
            .and_then(Value::as_str)
            == Some("rich_text");
        let Some(key) = setting.get("key").and_then(Value::as_str) else {
            continue;
        };
        if !rich {
            continue;
        }
        if let Some(value) = settings.get_mut(key) {
            sanitise(value);
        }
    }
    settings
}

/// Sanitises a rich text value, or each of its languages.
fn sanitise(value: &mut Value) {
    match value {
        Value::String(html) => *html = kampus_common::text::sanitise_rich(html),
        Value::Object(by_language) => by_language.values_mut().for_each(sanitise),
        _ => {}
    }
}
