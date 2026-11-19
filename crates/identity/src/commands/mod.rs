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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The commands identity offers on the command line: `kampus-server identity <command>`.

use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{Error, Result, SystemClock};
use kampus_module::{Command, CommandContext, CommandFuture};

use crate::{FixedPolicies, Identity, MemoryCourier, SecretKey};

/// The commands of identity.
pub const COMMANDS: &[Command] = &[
    Command {
        name: "purge",
        summary: "Remove expired one-time tokens and sessions, and sign-in attempts past their retention",
        arguments: &[],
        run: purge,
    },
    Command {
        name: "run-deletions",
        summary: "Delete the accounts whose grace period ended, leaving their tombstones",
        arguments: &[],
        run: run_deletions,
    },
];

/// The service, for a command.
fn identity(context: &CommandContext) -> Result<Identity> {
    let key = context
        .secret_key
        .as_deref()
        .ok_or_else(|| Error::validation("identity.secret_key_missing", Vec::new()))
        .and_then(SecretKey::from_base64)?;
    let clock = Arc::new(SystemClock);
    Ok(Identity::new(
        context.db.clone(),
        Access::new(context.db.clone(), clock.clone()),
        clock,
        Arc::new(FixedPolicies::default()),
        Arc::new(MemoryCourier::default()),
        key,
    ))
}

/// Runs the purge.
fn purge<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let purged = identity(context)?.purge().await?;
        Ok(vec![format!(
            "{} one-time token(s), {} session(s) and {} sign-in attempt(s) removed",
            purged.tokens, purged.sessions, purged.attempts
        )])
    })
}

/// Runs the deletions that are due.
fn run_deletions<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let deleted = identity(context)?.run_due_deletions().await?;
        Ok(vec![format!("{deleted} account(s) deleted")])
    })
}
