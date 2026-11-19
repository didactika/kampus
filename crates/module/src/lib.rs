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

//! The module contract of Kampus: every module crate exposes one [`Module`] in
//! `src/module.rs`, and the server reads it to install the module's schema and to synchronise what it
//! declares. Nothing is discovered from file names; the standard layout only tells developers where each
//! declaration lives.

pub mod context;
pub mod descriptor;
pub mod http;
pub mod permission;

pub use context::ServiceContext;
pub use descriptor::{
    Argument, Command, CommandContext, CommandFuture, Event, EventDeclaration, EventRun, JobDeclaration,
    JobFuture, JobRun, ListenerDeclaration, Module, PreferenceDeclaration, ScheduleDeclaration,
    SettingDeclaration, ValueKind,
};
pub use descriptor::{FieldKindDeclaration, FlagDeclaration, TermDeclaration, TermForms};
pub use permission::{
    PermissionDeclaration, PermissionKind, Risk, RoleTemplateDeclaration, ScopeKind, Value,
};
