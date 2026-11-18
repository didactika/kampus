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

//! The descriptor of a module: what it adds to the platform, as the server reads it.

use kampus_db::{ModuleSteps, Step};

use crate::permission::{PermissionDeclaration, RoleTemplateDeclaration};

/// What a module adds to the platform. Each kind of contribution has one field; a new kind is added here
/// by the core module that introduces it, together with the folder it lives in and its documentation.
#[derive(Debug, Clone, Copy)]
pub struct Module {
    /// The name of the module, also the logical name of its database schema (`identity`).
    pub name: &'static str,
    /// What the module does, in one sentence.
    pub summary: &'static str,
    /// The upgrade steps of its schema, from `src/db/steps.rs` (DEC-082).
    pub steps: &'static [Step],
    /// The permissions it declares, from `src/permissions/mod.rs` (RN-021).
    pub permissions: &'static [PermissionDeclaration],
    /// The values it gives in role templates, from `src/permissions/templates.rs`; the values of every
    /// module for one template are merged.
    pub templates: &'static [RoleTemplateDeclaration],
}

impl Module {
    /// The upgrade steps of the module, as the upgrade runner reads them.
    #[must_use]
    pub const fn schema(&self) -> ModuleSteps {
        ModuleSteps {
            module: self.name,
            steps: self.steps,
        }
    }
}
