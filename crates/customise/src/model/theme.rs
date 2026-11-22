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

//! Themes: the few tokens a scope sets (seed colours, typefaces, size, corners, density, motion and colour
//! scheme), merged from the platform down to the scope read, and the colour roles of a light and a dark
//! scheme grown from them, as the client applies them. A role keeps the seed chosen when it is accessible,
//! and the nearest accessible tone of its palette otherwise, so every text keeps WCAG 2.2 AA contrast on
//! its background whatever the seeds.

use std::collections::BTreeMap;
use std::fmt::Write;

use serde::Serialize;
use serde_json::Value;

use super::Origin;
use super::colour::{Rgb, WHITE};
use super::palette::Palette;

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
    /// Its name.
    pub name: &'static str,
    /// Its kind.
    pub kind: TokenKind,
    /// Its value when no scope sets it.
    pub default: &'static str,
}

/// A seed colour.
const fn seed(name: &'static str, default: &'static str) -> Token {
    Token {
        name,
        kind: TokenKind::Colour,
        default,
    }
}

/// The colours with a role of their own: each grows a palette and four roles (itself, the text on it, its
/// container and the text on that).
pub const COLOURS: [&str; 7] = [
    "primary",
    "secondary",
    "accent",
    "danger",
    "success",
    "warning",
    "info",
];

/// Every token of a theme. The defaults are Didactika's identity.
pub const TOKENS: &[Token] = &[
    seed("brand", "#00555a"),
    seed("primary", "#0f766e"),
    seed("secondary", "#475569"),
    seed("accent", "#06b6d4"),
    seed("neutral", "#64748b"),
    seed("danger", "#b91c1c"),
    seed("success", "#15803d"),
    seed("warning", "#b45309"),
    seed("info", "#0369a1"),
    Token {
        name: "font_family",
        kind: TokenKind::FontFamily,
        default: "\"Inter Variable\", Inter, system-ui, -apple-system, \"Segoe UI\", sans-serif",
    },
    Token {
        name: "font_family_brand",
        kind: TokenKind::FontFamily,
        default: "\"Inter Variable\", Inter, system-ui, -apple-system, \"Segoe UI\", sans-serif",
    },
    Token {
        name: "font_size",
        kind: TokenKind::Pixels { min: 12, max: 24 },
        default: "16",
    },
    Token {
        name: "radius",
        kind: TokenKind::Pixels { min: 0, max: 28 },
        default: "12",
    },
    Token {
        name: "density",
        kind: TokenKind::Choice(&["comfortable", "compact"]),
        default: "comfortable",
    },
    Token {
        name: "motion",
        kind: TokenKind::Choice(&["full", "reduced"]),
        default: "full",
    },
    Token {
        name: "scheme",
        kind: TokenKind::Choice(&["system", "light", "dark"]),
        default: "system",
    },
];

/// The smallest contrast of a text on its background, WCAG 2.2 AA for normal text.
pub const MIN_CONTRAST: f64 = 4.5;

/// The smallest contrast of the edge of a control on its background, WCAG 2.2 AA for non-text contrast.
pub const MIN_EDGE_CONTRAST: f64 = 3.0;

/// The surfaces text is read on.
const SURFACES: &[&str] = &[
    "surface",
    "surface-container-lowest",
    "surface-container-low",
    "surface-container",
    "surface-container-high",
    "surface-container-highest",
];

/// The colour roles of one scheme, light or dark, by name (`primary`, `on-primary-container`…).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Scheme(pub BTreeMap<String, String>);

/// The theme of a scope: every token with its value and where it came from, and the colour roles grown
/// from them.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct Theme {
    /// Every token, by name.
    pub tokens: BTreeMap<String, Value>,
    /// Where each token came from.
    pub origins: BTreeMap<String, Origin>,
    /// The colour roles of the light scheme.
    pub light: Scheme,
    /// The colour roles of the dark scheme.
    pub dark: Scheme,
}

/// Whether a scheme is light or dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Dark text on light surfaces.
    Light,
    /// Light text on dark surfaces.
    Dark,
}

/// The text of a background: white or the darkest tone of a palette, whichever contrasts more.
fn text_on(background: Rgb, darkest: Rgb) -> Rgb {
    if WHITE.contrast(background) >= darkest.contrast(background) {
        WHITE
    } else {
        darkest
    }
}

/// The first colour that keeps the contrast of text on a background, or the last one.
fn first_accessible(candidates: &[Rgb], background: Rgb) -> Rgb {
    candidates
        .iter()
        .copied()
        .find(|c| c.contrast(background) >= MIN_CONTRAST)
        .unwrap_or_else(|| candidates.last().copied().unwrap_or(WHITE))
}

impl Scheme {
    /// One role.
    #[must_use]
    pub fn role(&self, name: &str) -> Option<Rgb> {
        self.0.get(name).and_then(|c| Rgb::parse(c))
    }

    /// The pairs every scheme keeps: the text or edge, its background and the least contrast between them.
    #[must_use]
    pub fn pairs() -> Vec<(String, String, f64)> {
        let mut pairs = Vec::new();
        for colour in COLOURS {
            pairs.push((format!("on-{colour}"), colour.to_owned(), MIN_CONTRAST));
            pairs.push((
                format!("on-{colour}-container"),
                format!("{colour}-container"),
                MIN_CONTRAST,
            ));
            pairs.push((colour.to_owned(), "surface".to_owned(), MIN_CONTRAST));
        }
        pairs.push(("on-brand".to_owned(), "brand".to_owned(), MIN_CONTRAST));
        for surface in SURFACES {
            pairs.push(("on-surface".to_owned(), (*surface).to_owned(), MIN_CONTRAST));
            pairs.push((
                "on-surface-variant".to_owned(),
                (*surface).to_owned(),
                MIN_CONTRAST,
            ));
            pairs.push(("outline".to_owned(), (*surface).to_owned(), MIN_EDGE_CONTRAST));
        }
        pairs.push((
            "inverse-on-surface".to_owned(),
            "inverse-surface".to_owned(),
            MIN_CONTRAST,
        ));
        pairs.push((
            "inverse-primary".to_owned(),
            "inverse-surface".to_owned(),
            MIN_CONTRAST,
        ));
        pairs
    }

    /// The pairs of this scheme that lose their contrast, with the contrast they have.
    #[must_use]
    pub fn shortfalls(&self) -> Vec<(String, String, f64)> {
        Self::pairs()
            .into_iter()
            .filter_map(|(text, background, least)| {
                let ratio = self.role(&text)?.contrast(self.role(&background)?);
                (ratio < least).then_some((text, background, ratio))
            })
            .collect()
    }
}

impl Theme {
    /// A theme of these tokens and origins, with its colour roles grown from the tokens.
    #[must_use]
    pub fn new(tokens: BTreeMap<String, Value>, origins: BTreeMap<String, Origin>) -> Self {
        let light = Self::scheme(&tokens, Mode::Light);
        let dark = Self::scheme(&tokens, Mode::Dark);
        Self {
            tokens,
            origins,
            light,
            dark,
        }
    }

    /// A seed colour of the tokens, or its default.
    fn seed(tokens: &BTreeMap<String, Value>, name: &str) -> Rgb {
        tokens
            .get(name)
            .and_then(Value::as_str)
            .and_then(Rgb::parse)
            .or_else(|| {
                TOKENS
                    .iter()
                    .find(|t| t.name == name)
                    .and_then(|t| Rgb::parse(t.default))
            })
            .unwrap_or(WHITE)
    }

    /// The colour roles of one scheme.
    fn scheme(tokens: &BTreeMap<String, Value>, mode: Mode) -> Scheme {
        let neutral = Palette::neutral(Self::seed(tokens, "neutral"));
        let tone = |step| neutral.tone(step);
        let mut roles = BTreeMap::new();
        let mut set = |name: &str, colour: Rgb| {
            roles.insert(name.to_owned(), colour.to_string());
        };
        let (light, dark) = (tone(50), tone(900));
        let mut primaries = (WHITE, WHITE);
        for colour in COLOURS {
            let seed = Self::seed(tokens, colour);
            let palette = Palette::of(seed);
            let t = |step| palette.tone(step);
            let on_light = first_accessible(&[seed, t(600), t(700), t(800), t(900), t(950)], light);
            let on_dark = first_accessible(&[t(300), t(200), t(100), t(50)], dark);
            if colour == "primary" {
                primaries = (on_light, on_dark);
            }
            let (role, container, on_container) = match mode {
                Mode::Light => (on_light, t(100), t(900)),
                Mode::Dark => (on_dark, t(800), t(100)),
            };
            set(colour, role);
            set(&format!("on-{colour}"), text_on(role, t(950)));
            set(&format!("{colour}-container"), container);
            set(&format!("on-{colour}-container"), on_container);
        }
        let brand = Self::seed(tokens, "brand");
        set("brand", brand);
        set("on-brand", text_on(brand, tone(950)));
        set("scrim", tone(950));
        match mode {
            Mode::Light => {
                set("surface", tone(50));
                set("surface-container-lowest", WHITE);
                set("surface-container-low", tone(100));
                set("surface-container", tone(100).halfway(tone(200)));
                set("surface-container-high", tone(200));
                set("surface-container-highest", tone(200).halfway(tone(300)));
                set("on-surface", tone(900));
                set("on-surface-variant", tone(600));
                set("outline", tone(500));
                set("outline-variant", tone(300));
                set("inverse-surface", tone(800));
                set("inverse-on-surface", tone(100));
                set("inverse-primary", primaries.1);
                set("shadow", Palette::of(Self::seed(tokens, "primary")).tone(950));
            }
            Mode::Dark => {
                set("surface", tone(900));
                set("surface-container-lowest", tone(950));
                set("surface-container-low", tone(900).halfway(tone(800)));
                set("surface-container", tone(800));
                set("surface-container-high", tone(800).halfway(tone(700)));
                set("surface-container-highest", tone(700));
                set("on-surface", tone(50));
                set("on-surface-variant", tone(300));
                set("outline", tone(400).halfway(tone(300)));
                set("outline-variant", tone(700));
                set("inverse-surface", tone(100));
                set("inverse-on-surface", tone(800));
                set("inverse-primary", primaries.0);
                set("shadow", tone(950));
            }
        }
        Scheme(roles)
    }

    /// The text of a token, without quotes for a string.
    fn text(&self, name: &str) -> String {
        match self.tokens.get(name) {
            Some(Value::String(text)) => text.clone(),
            Some(other) => other.to_string(),
            None => String::new(),
        }
    }

    /// A whole number token.
    fn number(&self, name: &str) -> i64 {
        self.tokens.get(name).and_then(Value::as_i64).unwrap_or_default()
    }

    /// The colour roles of a scheme as custom properties, each line indented.
    fn roles(scheme: &Scheme, indent: &str) -> String {
        scheme.0.iter().fold(String::new(), |mut css, (role, colour)| {
            let _ = writeln!(css, "{indent}--k-color-{role}: {colour};");
            css
        })
    }

    /// The theme as the custom properties of the design system of the client, which applies them without
    /// code: the roles of the scheme the scope prefers on the root, the other scheme when the person or
    /// their system asks for it, and the typefaces, size, corners, density and motion.
    #[must_use]
    pub fn css(&self) -> String {
        let scheme = self.text("scheme");
        let (first, second, other) = if scheme == "dark" {
            (&self.dark, &self.light, "light")
        } else {
            (&self.light, &self.dark, "dark")
        };
        let mut css = String::from(":root {\n");
        css.push_str(&Self::roles(first, "  "));
        let _ = writeln!(css, "  --k-font-family-plain: {};", self.text("font_family"));
        let _ = writeln!(
            css,
            "  --k-font-family-brand: {};",
            self.text("font_family_brand")
        );
        let _ = writeln!(
            css,
            "  --k-font-size-root: {}%;",
            self.number("font_size") * 100 / 16
        );
        let _ = writeln!(css, "  --k-radius: {}px;", self.number("radius"));
        let density = if self.text("density") == "compact" { -2 } else { 0 };
        let _ = writeln!(css, "  --k-density: {density};");
        let motion = u8::from(self.text("motion") != "reduced");
        let _ = writeln!(css, "  --k-motion: {motion};");
        let modes = match scheme.as_str() {
            "light" => "light",
            "dark" => "dark",
            _ => "light dark",
        };
        let _ = writeln!(css, "  color-scheme: {modes};\n}}");
        if scheme == "system" {
            let _ = writeln!(
                css,
                "@media (prefers-color-scheme: dark) {{\n  :root:not([data-theme=\"light\"]) {{\n{}  }}\n}}",
                Self::roles(second, "    ")
            );
        }
        let _ = writeln!(
            css,
            ":root[data-theme=\"{other}\"] {{\n{}  color-scheme: {other};\n}}",
            Self::roles(second, "  ")
        );
        css
    }
}
