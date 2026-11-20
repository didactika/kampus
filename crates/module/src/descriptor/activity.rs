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

//! What a module offers to activities: the types of activity and material it provides, each with the
//! settings its course modules keep. The activities module registers them, places course modules of them
//! in sections, and never reads what a type keeps of its own.

use crate::ValueKind;

/// Whether a type is something people do or something they read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    /// Something people do: it can have dates, attempts, submissions and grades.
    Activity,
    /// Something people read or open: a page, a file, a link.
    Material,
}

impl ActivityKind {
    /// The kind as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Material => "material",
        }
    }
}

/// A setting of a type of activity, kept for each course module of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivitySetting {
    /// Its key, unique in the type.
    pub key: &'static str,
    /// The kind of value it takes, with its default.
    pub kind: ValueKind,
}

/// A type of activity or material a module provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityTypeDeclaration {
    /// `<module>.<type>`, unique in the build.
    pub code: &'static str,
    /// Whether it is an activity or a material.
    pub kind: ActivityKind,
    /// What it is, for the people who add it.
    pub summary: &'static str,
    /// What it supports, such as `grading` or `exceptions`.
    pub features: &'static [&'static str],
    /// The settings each of its course modules keeps.
    pub settings: &'static [ActivitySetting],
}
