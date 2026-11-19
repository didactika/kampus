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

//! Customisation packages: what a scope sets of the kinds chosen, exported to be imported into another
//! scope or campus, and the imports waiting to be applied with the differences they would make.

use std::collections::BTreeMap;

use kampus_access::ScopeId;
use kampus_common::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::FieldSpec;
use super::layout::{MenuItem, Placement};
use super::rule::RuleSpec;

kampus_common::define_id! {
    /// Identifies an import of a package.
    pub struct ImportId;
}

/// The format of the packages this release writes and reads.
pub const FORMAT: u16 = 1;

/// A kind of customisation a package holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    /// Words of terms.
    Terms,
    /// Values of feature flags.
    Flags,
    /// Custom fields, without their values.
    Fields,
    /// Layouts of pages.
    Layouts,
    /// Navigation menus.
    Menus,
    /// The theme.
    Theme,
    /// Rules.
    Rules,
}

/// The word a scope gives a term in a language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTerm {
    /// The term.
    pub code: String,
    /// The language.
    pub language: String,
    /// The word for one.
    pub singular: String,
    /// The word for several.
    pub plural: String,
    /// Whether the scopes below keep it.
    #[serde(default)]
    pub locked: bool,
}

/// The value a scope gives a feature flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFlag {
    /// The flag.
    pub code: String,
    /// Whether the feature is on.
    pub enabled: bool,
    /// Whether the scopes below keep it.
    #[serde(default)]
    pub locked: bool,
}

/// A custom field a scope defines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageField {
    /// The kind of entity it belongs to.
    pub kind: String,
    /// The field.
    #[serde(flatten)]
    pub spec: FieldSpec,
}

/// The layout a scope gives a page kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageLayout {
    /// The page kind.
    pub page_kind: String,
    /// Its widgets, in order.
    pub placements: Vec<Placement>,
    /// Whether the scopes below keep it.
    #[serde(default)]
    pub locked: bool,
}

/// The menu a scope gives a kind of menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageMenu {
    /// The kind of menu.
    pub kind: String,
    /// Its items, in order.
    pub items: Vec<MenuItem>,
    /// Whether the scopes below keep it.
    #[serde(default)]
    pub locked: bool,
}

/// The tokens a scope sets in its theme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageTheme {
    /// The tokens, by name.
    pub tokens: BTreeMap<String, Value>,
    /// Whether the scopes below keep them.
    #[serde(default)]
    pub locked: bool,
}

/// A package of customisations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// The format it is written in.
    pub format: u16,
    /// The words of terms.
    #[serde(default)]
    pub terms: Vec<PackageTerm>,
    /// The values of feature flags.
    #[serde(default)]
    pub flags: Vec<PackageFlag>,
    /// The custom fields.
    #[serde(default)]
    pub fields: Vec<PackageField>,
    /// The layouts.
    #[serde(default)]
    pub layouts: Vec<PackageLayout>,
    /// The menus.
    #[serde(default)]
    pub menus: Vec<PackageMenu>,
    /// The theme.
    #[serde(default)]
    pub theme: Option<PackageTheme>,
    /// The rules, which the importer owns once applied.
    #[serde(default)]
    pub rules: Vec<RuleSpec>,
}

impl Package {
    /// An empty package.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            format: FORMAT,
            terms: Vec::new(),
            flags: Vec::new(),
            fields: Vec::new(),
            layouts: Vec::new(),
            menus: Vec::new(),
            theme: None,
            rules: Vec::new(),
        }
    }
}

/// A package as exported, with the checksum of its contents.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Exported {
    /// The package.
    pub package: Package,
    /// The SHA-256 of the package as written, in hexadecimal.
    pub checksum: String,
}

/// What an import would do to one customisation of the scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Difference {
    /// The kind of customisation.
    pub kind: PackageKind,
    /// What it is, such as the code of a term and its language.
    pub key: String,
    /// What the scope sets now; nothing when the import adds it.
    pub before: Option<Value>,
    /// What the scope will set.
    pub after: Value,
}

/// An import of a package into a scope.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Import {
    /// The import.
    pub id: ImportId,
    /// The scope it goes into.
    pub scope: ScopeId,
    /// The checksum of the package.
    pub checksum: String,
    /// What it adds or changes; what the scope already sets the same way is left out.
    pub differences: Vec<Difference>,
    /// When it was applied, if it was.
    pub applied_at: Option<Timestamp>,
}
