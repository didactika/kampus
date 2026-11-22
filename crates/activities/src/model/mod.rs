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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of activities: a course module, one activity or material placed in a section, and what
//! goes to the recycle bin.

use std::collections::BTreeMap;

use kampus_access::ScopeId;
use kampus_common::Timestamp;
use kampus_courses::{SectionId, Visibility};
use serde::{Deserialize, Serialize};
use serde_json::Value;

kampus_common::define_id! {
    /// Identifies a course module.
    pub struct ModuleId;
}

/// Where a course module stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModuleState {
    /// In its section.
    Active,
    /// In the recycle bin of its course.
    InBin,
}

/// A course module: one activity or material placed in a section, with a scope of its own under it.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct Module {
    /// The course module.
    pub id: ModuleId,
    /// Its scope, under the scope of its section.
    pub scope: ScopeId,
    /// Its section.
    pub section: SectionId,
    /// Its type, such as `activities.page`.
    #[serde(rename = "type")]
    pub type_code: String,
    /// Its title.
    pub title: String,
    /// Its place among the course modules of its section.
    pub sort_key: String,
    /// Whether it is shown, hidden or shown for a while.
    pub visibility: Visibility,
    /// Whether it is listed on the course page; separate from whether people may open it.
    pub on_course_page: bool,
    /// What its type keeps for it, each of its declared kind.
    pub settings: BTreeMap<String, Value>,
    /// Where it stands.
    pub state: ModuleState,
    /// Whether the person who asked meets its access conditions; when not, what the course module holds is
    /// left out.
    pub available: bool,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// A course module to add, or the new values of one.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
pub struct ModuleFields {
    /// Its title.
    pub title: String,
    /// Whether it is listed on the course page.
    #[serde(default = "listed")]
    pub on_course_page: bool,
    /// Whether it is shown, hidden or shown for a while.
    #[serde(default = "shown")]
    pub visibility: Visibility,
    /// What its type keeps for it; the defaults fill what is left out.
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
}

/// Course modules are listed unless said otherwise.
const fn listed() -> bool {
    true
}

/// Course modules are shown unless said otherwise.
const fn shown() -> Visibility {
    Visibility::Shown
}

/// A type of activity or material, as registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct ActivityType {
    /// Its code.
    pub code: String,
    /// `activity` or `material`.
    pub kind: String,
    /// What it is.
    pub summary: String,
    /// What it supports.
    pub features: Vec<String>,
    /// Whether course modules of it may be added.
    pub enabled: bool,
}

/// A course module in the recycle bin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct BinItem {
    /// The course module.
    pub module: ModuleId,
    /// Its title.
    pub title: String,
    /// Its type.
    #[serde(rename = "type")]
    pub type_code: String,
    /// When it was deleted.
    pub deleted_at: Timestamp,
    /// When it is removed for good.
    pub expires_at: Timestamp,
    /// Why removing it failed, which keeps it in the bin until someone retries or restores it.
    pub error: Option<String>,
}

/// How the children of a node of a tree of conditions combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    /// Every child holds.
    All,
    /// At least one child holds.
    Any,
    /// The only child does not hold.
    Not,
    /// A condition of a kind, with its parameters.
    Leaf,
}

impl Operator {
    /// The stored name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Any => "any",
            Self::Not => "not",
            Self::Leaf => "leaf",
        }
    }

    /// The operator of a stored name; an unknown one reads as a leaf, which a tree treats as unmet.
    #[must_use]
    pub fn read(name: &str) -> Self {
        match name {
            "all" => Self::All,
            "any" => Self::Any,
            "not" => Self::Not,
            _ => Self::Leaf,
        }
    }
}

/// A node of a tree of access conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConditionNode {
    /// How it combines its children, or `leaf`.
    pub operator: Operator,
    /// Its children, for `all`, `any` and `not`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ConditionNode>,
    /// The kind of a leaf, such as `activities.date`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// The parameters of a leaf, as its kind takes them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Set on a leaf whose kind is unknown or that lost what it depends on: it never holds.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub broken: bool,
}

/// The access conditions of a course module or a section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Conditions {
    /// Whether people who do not meet them still see it listed, without its content.
    #[serde(default = "listed")]
    pub show_when_unmet: bool,
    /// The tree.
    pub tree: ConditionNode,
}

/// What access conditions are on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restricted {
    /// A course module.
    Module(ModuleId),
    /// A section, and everything in it and in its subsections.
    Section(SectionId),
}

/// Whether a person may open a course module, and whether they see it at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Availability {
    /// They meet every condition on it and on the sections around it.
    pub available: bool,
    /// They see it listed: it is available, or every condition they miss shows it.
    pub shown: bool,
}
