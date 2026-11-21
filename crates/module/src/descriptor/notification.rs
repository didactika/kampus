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

//! What a module offers to notifications: the types of notification it declares, each with the channels it
//! is delivered through by default and whether people may turn it off. The notifications module creates
//! them from the events of the modules and delivers them according to people's preferences.

/// A type of notification a module declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationTypeDeclaration {
    /// `<module>.<what_happened>`, unique in the build.
    pub code: &'static str,
    /// What it tells people, for the settings screens.
    pub summary: &'static str,
    /// The channels it is delivered through unless a person chooses otherwise.
    pub channels: &'static [&'static str],
    /// Whether it cannot be turned off, only routed to another channel.
    pub required: bool,
}
