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

mod activity;
mod command;
mod condition;
mod customise;
mod event;
mod file;
mod job;
mod notification;
mod operation;
mod preference;
mod privacy;
mod progress;
mod rule;
mod setting;
mod value;

pub use activity::{ActivityKind, ActivitySetting, ActivityTypeDeclaration};
pub use command::{Argument, Command, CommandContext, CommandFuture};
pub use condition::{
    ConditionCheck, ConditionFuture, ConditionKindDeclaration, ConditionTarget, ConditionWrite,
    nothing_depends,
};
pub use customise::{
    FieldKindDeclaration, FlagDeclaration, MenuKindDeclaration, PageKindDeclaration, TermDeclaration,
    TermForms, WidgetDeclaration, WidgetSetting,
};
pub use event::{Event, EventDeclaration, EventRun, ListenerDeclaration};
pub use file::{FileAreaDeclaration, FileRead, ReadFuture};
pub use job::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};
pub use notification::NotificationTypeDeclaration;
pub use operation::{Content, Method, Operation, SchemaOf, schema_of};
pub use preference::PreferenceDeclaration;
pub use privacy::{PrivacyFuture, PrivacyHandlerDeclaration, PrivacyRun};
pub use progress::ProgressRuleDeclaration;
pub use rule::{RuleActionDeclaration, RuleActionRun, RuleParameter};
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
    /// The words of its domain a campus may rename, from `src/settings/customise.rs`.
    pub terms: &'static [TermDeclaration],
    /// The features a scope may turn off, from `src/settings/customise.rs`.
    pub flags: &'static [FlagDeclaration],
    /// The kinds of entity that accept custom fields, from `src/settings/customise.rs`.
    pub field_kinds: &'static [FieldKindDeclaration],
    /// The kinds of page it shows, with their regions, from `src/settings/customise.rs`.
    pub page_kinds: &'static [PageKindDeclaration],
    /// The widgets it offers for layouts, from `src/settings/customise.rs`.
    pub widgets: &'static [WidgetDeclaration],
    /// The kinds of navigation menu it shows, from `src/settings/customise.rs`.
    pub menu_kinds: &'static [MenuKindDeclaration],
    /// The actions it offers to rules, from `src/settings/customise.rs`.
    pub rule_actions: &'static [RuleActionDeclaration],
    /// The areas where it keeps files, from `src/settings/files.rs`.
    pub file_areas: &'static [FileAreaDeclaration],
    /// The types of activity and material it provides, from `src/settings/activities.rs`.
    pub activity_types: &'static [ActivityTypeDeclaration],
    /// The types of notification it declares, from `src/settings/notifications.rs`.
    pub notification_types: &'static [NotificationTypeDeclaration],
    /// The personal data it keeps and how to export it, from `src/privacy/mod.rs`.
    pub privacy_handlers: &'static [PrivacyHandlerDeclaration],
    /// The kinds of access condition it provides, from `src/settings/conditions.rs`.
    pub condition_kinds: &'static [ConditionKindDeclaration],
    /// The rules of progress its activity types offer, from `src/settings/activities.rs`.
    pub progress_rules: &'static [ProgressRuleDeclaration],
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
    /// The operations its routes answer, with the schemas of what they take and answer, from
    /// `src/http/mod.rs`: one list per file of routes.
    pub operations: &'static [&'static [Operation]],
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
