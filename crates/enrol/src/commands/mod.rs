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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The commands enrolment offers on the command line: `kampus-server enrol <command>`.

use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{Error, Result, SystemClock};
use kampus_courses::Courses;
use kampus_identity::{FixedPolicies, Identity, LogCourier, PasswordCost, PasswordHasher, SecretKey};
use kampus_module::{Command, CommandContext, CommandFuture};
use kampus_users::Users;

use crate::{Enrol, Participation};

/// The commands of enrolment.
pub const COMMANDS: &[Command] = &[
    Command {
        name: "sync",
        summary: "Bring the enrolments of every system group instance in step with its group",
        arguments: &[],
        run: sync,
    },
    Command {
        name: "run-expiry",
        summary: "Apply the expiry rule to the enrolments whose end has passed",
        arguments: &[],
        run: run_expiry,
    },
];

/// The service, for a command.
fn enrol(context: &CommandContext) -> Result<Enrol> {
    let key = context
        .secret_key
        .as_deref()
        .ok_or_else(|| Error::validation("enrol.secret_key_missing", Vec::new()))
        .and_then(SecretKey::from_base64)?;
    let clock = Arc::new(SystemClock);
    let db = context.db.clone();
    let access = Access::new(db.clone(), clock.clone());
    let identity = Identity::new(
        db.clone(),
        access.clone(),
        clock.clone(),
        Arc::new(FixedPolicies::default()),
        Arc::new(LogCourier),
        key,
    );
    let users = Users::new(db.clone(), access.clone(), identity.clone(), Vec::new());
    let courses = Courses::new(
        db.clone(),
        access.clone(),
        clock.clone(),
        Arc::new(Participation::new(db.clone(), clock.clone())),
    );
    Ok(Enrol::new(
        db,
        access,
        courses,
        identity,
        users,
        clock,
        PasswordHasher::new(PasswordCost::default()),
    ))
}

/// Runs the sync of every system group instance.
fn sync<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let synced = enrol(context)?.sync_all().await?;
        Ok(vec![format!(
            "{} enrolment(s) added and {} removed",
            synced.added, synced.removed
        )])
    })
}

/// Runs the expiry.
fn run_expiry<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let expired = enrol(context)?.run_expiry().await?;
        Ok(vec![format!(
            "{} enrolment(s) suspended, {} removed and {} kept without their role",
            expired.suspended, expired.removed, expired.kept_without_role
        )])
    })
}
