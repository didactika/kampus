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

mod command;
mod customise;
mod event;
mod job;
mod preference;
mod setting;
mod value;

pub use command::{Argument, Command, CommandContext, CommandFuture};
pub use customise::{FieldKindDeclaration, FlagDeclaration, TermDeclaration, TermForms};
pub use event::{Event, EventDeclaration, EventRun, ListenerDeclaration};
pub use job::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};
pub use preference::PreferenceDeclaration;
pub use setting::SettingDeclaration;
pub use value::ValueKind;

use kampus_db::{ModuleSteps, Step};

use crate::permission::{PermissionDeclaration, RoleTemplateDeclaration};

/// What a module adds to the platform. Each kind of contribution has one field; a new kind is added here
/// by the core module that introduces it, together with the folder it lives in and its documentation.
#[derive(Debug, Clone, Copy)]
pub struct Module {
    /// What it says about itself: version, maturity, the releases it supports, what it requires, its icon.
    pub manifest: kampus_common::component::Manifest,
    /// The name of the module, also the logical name of its database schema (`identity`).
    pub name: &'static str,
    /// What the module does, in one sentence.
    pub summary: &'static str,
    /// The upgrade steps of its schema, from `src/db/steps.rs`.
    pub steps: &'static [Step],
    /// The permissions it declares, from `src/permissions/mod.rs`.
    pub permissions: &'static [PermissionDeclaration],
    /// The values it gives in role templates, from `src/permissions/templates.rs`; the values of every
    /// module for one template are merged.
    pub templates: &'static [RoleTemplateDeclaration],
    /// The plugin types it defines: their plugins are named `<type>_<name>` and live in
    /// `crates/<this module>/<type>/<name>`.
    pub plugin_types: &'static [&'static str],
    /// The commands it offers on the command line, from `src/commands/mod.rs`.
    pub commands: &'static [Command],
    /// The per-person preferences it declares, from `src/settings/preferences.rs`.
    pub preferences: &'static [PreferenceDeclaration],
    /// The settings it declares, kept per scope, from `src/settings/mod.rs`.
    pub settings: &'static [SettingDeclaration],
    /// The words of its domain a campus may rename, from `src/settings/`.
    pub terms: &'static [TermDeclaration],
    /// The features a scope may turn off, from `src/settings/`.
    pub flags: &'static [FlagDeclaration],
    /// The kinds of entity that accept custom fields.
    pub field_kinds: &'static [FieldKindDeclaration],
    /// The events it publishes, from `src/events/mod.rs`.
    pub events: &'static [EventDeclaration],
    /// Its reactions to events, from `src/observers/mod.rs`.
    pub listeners: &'static [ListenerDeclaration],
    /// The kinds of background job it runs, from `src/jobs/mod.rs`.
    pub jobs: &'static [JobDeclaration],
    /// The schedules it declares, with their default rules, from `src/jobs/mod.rs`.
    pub schedules: &'static [ScheduleDeclaration],
    /// Its HTTP routes, from `src/http/mod.rs`, built with the services the server composed.
    pub routes: Option<fn(&crate::ServiceContext) -> axum::Router>,
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
