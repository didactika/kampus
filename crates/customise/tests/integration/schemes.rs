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

//! The colour roles of a theme without a database: whatever the seeds, every text keeps its contrast; an
//! accessible seed is kept as it is, Didactika's palettes are used as they are, and the defaults are the
//! tokens of the design system of the web client.

use std::collections::BTreeMap;

use kampus_customise::Theme;
use kampus_customise::model::colour::{Lch, Rgb};
use kampus_customise::model::palette::{Palette, TEAL};
use kampus_customise::model::theme::{COLOURS, TOKENS, TokenKind};
use serde_json::{Value, json};

/// The default of every token, with some of them changed.
fn tokens(changes: &[(&str, Value)]) -> BTreeMap<String, Value> {
    let mut tokens: BTreeMap<String, Value> = TOKENS
        .iter()
        .map(|t| {
            let value = match t.kind {
                TokenKind::Pixels { .. } => json!(t.default.parse::<i64>().unwrap()),
                _ => json!(t.default),
            };
            (t.name.to_owned(), value)
        })
        .collect();
    for (name, value) in changes {
        tokens.insert((*name).to_owned(), value.clone());
    }
    tokens
}

/// The theme of these tokens.
fn theme(changes: &[(&str, Value)]) -> Theme {
    Theme::new(tokens(changes), BTreeMap::new())
}

#[test]
fn every_seed_grows_schemes_that_keep_their_contrast() {
    let mut seeds = Vec::new();
    for hue in (0..360).step_by(20) {
        for lightness in [0.25, 0.45, 0.65, 0.85, 0.97] {
            for chroma in [0.0, 0.08, 0.2, 0.35] {
                seeds.push(
                    Lch {
                        lightness,
                        chroma,
                        hue: f64::from(hue),
                    }
                    .rgb(),
                );
            }
        }
    }
    for seed in seeds {
        let colour = json!(seed.to_string());
        let mut all: Vec<(&str, Value)> = COLOURS.iter().map(|c| (*c, colour.clone())).collect();
        all.push(("brand", colour.clone()));
        for neutral in [None, Some(colour.clone())] {
            let mut changes = all.clone();
            if let Some(neutral) = neutral {
                changes.push(("neutral", neutral));
            }
            let theme = theme(&changes);
            assert_eq!(theme.light.shortfalls(), vec![], "light scheme of {seed}");
            assert_eq!(theme.dark.shortfalls(), vec![], "dark scheme of {seed}");
        }
    }
}

#[test]
fn an_accessible_seed_is_kept_and_didactikas_palettes_are_used_as_they_are() {
    let didactika = theme(&[]);
    assert_eq!(didactika.light.0["primary"], "#0f766e");
    assert_eq!(didactika.light.0["primary-container"], "#ccfbf1");
    assert_eq!(didactika.dark.0["primary"], "#5eead4");
    assert_eq!(didactika.dark.0["primary-container"], "#115e59");
    assert_eq!(
        didactika.light.0["accent"], "#0e7490",
        "the accent is too light for text, so a darker cyan"
    );
    assert_eq!(didactika.light.0["surface"], "#f8fafc");
    assert_eq!(didactika.dark.0["surface"], "#0f172a");
    assert_eq!(didactika.light.0["brand"], "#00555a");
    assert_eq!(didactika.light.0["on-brand"], "#ffffff");

    let blue = theme(&[("primary", json!("#1d4ed8"))]);
    assert_eq!(blue.light.0["primary"], "#1d4ed8");
    let seed = Rgb::parse("#1d4ed8").unwrap();
    let grown = Palette::of(seed);
    assert!(grown.0.contains(&seed), "the seed is a tone of its palette");
    assert!(
        grown
            .0
            .windows(2)
            .all(|pair| pair[0].luminance() > pair[1].luminance()),
        "the tones go from light to dark"
    );
    assert_eq!(Palette::of(Rgb::parse("#14b8a6").unwrap()), TEAL);
}

#[test]
fn the_css_puts_the_scheme_the_scope_prefers_first_and_the_other_on_request() {
    let system = theme(&[]).css();
    assert!(system.contains("color-scheme: light dark;"));
    assert!(system.contains("@media (prefers-color-scheme: dark)"));
    assert!(system.contains(":root[data-theme=\"dark\"]"));
    assert!(system.contains("--k-font-size-root: 100%;"));
    assert!(system.contains("--k-radius: 12px;"));
    assert!(system.contains("--k-density: 0;"));
    assert!(system.contains("--k-motion: 1;"));

    let dark = theme(&[
        ("scheme", json!("dark")),
        ("density", json!("compact")),
        ("motion", json!("reduced")),
    ])
    .css();
    let root = &dark[..dark.find('}').unwrap()];
    assert!(root.contains("--k-color-surface: #0f172a;"), "{dark}");
    assert!(!dark.contains("@media"));
    assert!(dark.contains(":root[data-theme=\"light\"]"));
    assert!(dark.contains("--k-density: -2;"));
    assert!(dark.contains("--k-motion: 0;"));
}

/// The colour roles of a scheme of the design system of the web client, resolved from its token files.
fn design_system(mode: &str) -> BTreeMap<String, String> {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/../../web/ui/tokens/");
    let read = |file: &str| -> Value {
        serde_json::from_str(&std::fs::read_to_string(format!("{folder}{file}")).unwrap()).unwrap()
    };
    let reference = read("reference.tokens.json");
    let light = read("light.tokens.json");
    let roles = if mode == "light" {
        light["color"].clone()
    } else {
        let mut roles = light["color"].clone();
        for (name, value) in read("dark.tokens.json")["color"].as_object().unwrap() {
            roles[name] = value.clone();
        }
        roles
    };
    roles
        .as_object()
        .unwrap()
        .iter()
        .filter(|(name, _)| !name.starts_with('$'))
        .map(|(name, token)| {
            let value = token["$value"].as_str().unwrap();
            let resolved = value.strip_prefix('{').map_or_else(
                || value.to_owned(),
                |path| {
                    let mut node = &reference;
                    for key in path.trim_end_matches('}').split('.') {
                        node = &node[key];
                    }
                    node["$value"].as_str().unwrap().to_owned()
                },
            );
            (name.clone(), resolved)
        })
        .collect()
}

#[test]
fn the_default_theme_is_the_design_system_of_the_web_client() {
    let didactika = theme(&[]);
    assert_eq!(design_system("light"), didactika.light.0);
    assert_eq!(design_system("dark"), didactika.dark.0);
}
