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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleState {
    /// In its section.
    Active,
    /// In the recycle bin of its course.
    InBin,
}

/// A course module: one activity or material placed in a section, with a scope of its own under it.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// The version, for `If-Match`.
    pub version: i32,
}

/// A course module to add, or the new values of one.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
