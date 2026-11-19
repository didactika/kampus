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
// @package   kampus-macros
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Macros of Kampus that read the files of the crate that uses them, at compile time.
//!
//! Use them through `kampus-common`, which re-exports them.

use std::path::Path;

use proc_macro::{TokenStream, TokenTree};

/// Where a component keeps its icon unless its manifest says otherwise.
const ICON: &str = "assets/icon.svg";

/// Writes the manifest of the calling crate: package, version, description and licence from its
/// `Cargo.toml`, the icon from `assets/icon.svg` when the file exists, and the optional fields given by
/// name. An `icon` given in the input replaces the default one.
#[proc_macro]
pub fn manifest(input: TokenStream) -> TokenStream {
    let gives_icon = gives(&input, "icon");
    let mut fields = input.to_string();
    if !fields.trim().is_empty() && !fields.trim_end().ends_with(',') {
        fields.push(',');
    }
    let icon = std::env::var("CARGO_MANIFEST_DIR")
        .ok()
        .map(|dir| Path::new(&dir).join(ICON))
        .filter(|path| !gives_icon && path.is_file())
        .map_or_else(String::new, |path| {
            format!(
                "icon: ::core::option::Option::Some(include_str!({:?})),",
                path.display().to_string()
            )
        });
    let code = format!(
        "{{
            #[allow(clippy::needless_update, reason = \"a manifest may give every optional field\")]
            let manifest = ::kampus_common::component::Manifest {{
                package: env!(\"CARGO_PKG_NAME\"),
                version: env!(\"CARGO_PKG_VERSION\"),
                summary: env!(\"CARGO_PKG_DESCRIPTION\"),
                license: env!(\"CARGO_PKG_LICENSE\"),
                {fields}
                {icon}
                ..::kampus_common::component::Manifest::DEFAULTS
            }};
            manifest
        }}"
    );
    code.parse().unwrap_or_else(|_| {
        "compile_error!(\"kampus_common::manifest! could not read its fields\")"
            .parse()
            .unwrap_or_default()
    })
}

/// Whether the input of the macro gives a field: its name followed by a colon at the top level.
fn gives(input: &TokenStream, field: &str) -> bool {
    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();
    tokens.windows(2).any(|pair| {
        matches!(&pair[0], TokenTree::Ident(name) if name.to_string() == field)
            && matches!(&pair[1], TokenTree::Punct(p) if p.as_char() == ':')
    })
}
