// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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
// @package   kampus-db
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The database port of Kampus, PostgreSQL first (DEC-009, DEC-065).
//!
//! - [`Prefix`]: the installation prefix of every physical name (DEC-039).
//! - [`Step`] and [`ModuleSteps`]: the ordered, immutable upgrade steps of each module (DEC-080).
//! - [`Upgrader`]: applies every pending step of every module in one run, from any supported release
//!   (DEC-082), and refuses a database that is newer than the code or whose steps changed.
//! - [`Database`]: the connection pool with the prefix, and transactions.

mod database;
mod prefix;
mod step;
mod upgrade;

pub use database::{Database, Transaction};
pub use prefix::Prefix;
pub use step::{ModuleSteps, Step};
pub use upgrade::{Problem, UpgradePlan, UpgradeReport, Upgrader};

/// The logical name of the schema this crate owns: the ledger of applied steps and the installation.
pub const MODULE: &str = "db";
