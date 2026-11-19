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

//! Themes: design tokens set per scope, merged from the platform down to the scope read, and the schema
//! every token keeps to, with the colour pairs that must keep WCAG 2.2 AA contrast.

use std::collections::BTreeMap;
use std::fmt::Write;

use serde::Serialize;
use serde_json::Value;

use super::Origin;

/// The kind of value of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A colour as `#rrggbb`.
    Colour,
    /// A whole number of pixels within bounds.
    Pixels {
        /// The smallest.
        min: i64,
        /// The largest.
        max: i64,
    },
    /// A list of font families.
    FontFamily,
    /// One of these words.
    Choice(&'static [&'static str]),
}

/// A design token, with its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// Its name, which is also the name of its CSS custom property after `--kampus-`.
    pub name: &'static str,
    /// Its kind.
    pub kind: TokenKind,
    /// Its value when no scope sets it.
    pub default: &'static str,
}

/// A colour token.
const fn colour(name: &'static str, default: &'static str) -> Token {
    Token {
        name,
        kind: TokenKind::Colour,
        default,
    }
}

/// Every token of a theme.
pub const TOKENS: &[Token] = &[
    colour("primary", "#1d4ed8"),
    colour("on_primary", "#ffffff"),
    colour("secondary", "#475569"),
    colour("on_secondary", "#ffffff"),
    colour("surface", "#ffffff"),
    colour("on_surface", "#1f2937"),
    colour("background", "#f8fafc"),
    colour("on_background", "#111827"),
    colour("danger", "#b91c1c"),
    colour("on_danger", "#ffffff"),
    colour("link", "#1d4ed8"),
    Token {
        name: "font_family",
        kind: TokenKind::FontFamily,
        default: "system-ui, sans-serif",
    },
    Token {
        name: "font_size",
        kind: TokenKind::Pixels { min: 12, max: 24 },
        default: "16",
    },
    Token {
        name: "spacing",
        kind: TokenKind::Pixels { min: 2, max: 16 },
        default: "4",
    },
    Token {
        name: "radius",
        kind: TokenKind::Pixels { min: 0, max: 32 },
        default: "6",
    },
    Token {
        name: "motion",
        kind: TokenKind::Choice(&["full", "reduced"]),
        default: "full",
    },
];

/// The pairs of colours, text on its background, that keep a contrast of at least 4.5 to 1.
pub const CONTRAST_PAIRS: &[(&str, &str)] = &[
    ("on_primary", "primary"),
    ("on_secondary", "secondary"),
    ("on_surface", "surface"),
    ("on_background", "background"),
    ("on_danger", "danger"),
    ("link", "surface"),
    ("link", "background"),
];

/// The smallest contrast of a pair, WCAG 2.2 AA for normal text.
pub const MIN_CONTRAST: f64 = 4.5;

/// The theme of a scope: every token with its value and where it came from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Theme {
    /// Every token, by name.
    pub tokens: BTreeMap<String, Value>,
    /// Where each token came from.
    pub origins: BTreeMap<String, Origin>,
}

impl Theme {
    /// The tokens as CSS custom properties on `:root`, for the client to apply without code.
    #[must_use]
    pub fn css(&self) -> String {
        let mut css = String::from(":root {\n");
        for token in TOKENS {
            let value = self.tokens.get(token.name).map_or_else(String::new, |v| match v {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            });
            let unit = if matches!(token.kind, TokenKind::Pixels { .. }) {
                "px"
            } else {
                ""
            };
            let name = token.name.replace('_', "-");
            let _ = writeln!(css, "  --kampus-{name}: {value}{unit};");
        }
        css.push_str("}\n");
        css
    }
}
