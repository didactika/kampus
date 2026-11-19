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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The commands events offers on the command line, `kampus-server events <command>`: the operator's view of
//! deliveries and the audit log until their API arrives.

use kampus_common::{Error, FieldError, Result};
use kampus_module::{Argument, Command, CommandContext, CommandFuture};

use crate::Events;

/// The commands of events.
pub const COMMANDS: &[Command] = &[
    Command {
        name: "deliveries",
        summary: "List the latest deliveries in a state: pending, done or dead (the default)",
        arguments: &[Argument {
            name: "state",
            summary: "pending, done or dead",
            required: false,
        }],
        run: deliveries,
    },
    Command {
        name: "replay",
        summary: "Run the dead deliveries again, of one event or all",
        arguments: &[Argument {
            name: "event",
            summary: "The event; every dead delivery when absent",
            required: false,
        }],
        run: replay,
    },
    Command {
        name: "listener",
        summary: "Turn a listener off or on; while off its deliveries wait",
        arguments: &[
            Argument {
                name: "module",
                summary: "The module of the listener",
                required: true,
            },
            Argument {
                name: "name",
                summary: "The name of the listener",
                required: true,
            },
            Argument {
                name: "on-or-off",
                summary: "on or off",
                required: true,
            },
        ],
        run: listener,
    },
    Command {
        name: "audit",
        summary: "Show the latest entries of the audit log",
        arguments: &[],
        run: audit,
    },
];

/// The argument at a position.
fn argument<'a>(arguments: &'a [String], position: usize, name: &'static str) -> Result<&'a str> {
    arguments
        .get(position)
        .map(String::as_str)
        .ok_or_else(|| Error::validation("events.argument_missing", vec![FieldError::new(name, "required")]))
}

/// Lists deliveries.
fn deliveries<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let state = arguments.first().map_or("dead", String::as_str);
        let deliveries = Events::new(context.db.clone()).deliveries(state, 50).await?;
        Ok(deliveries
            .iter()
            .map(|d| {
                format!(
                    "{}  {:<32} {}.{:<24} attempts {}  {}",
                    d.event,
                    d.code,
                    d.module,
                    d.listener,
                    d.attempts,
                    d.last_error.as_deref().unwrap_or("")
                )
            })
            .collect())
    })
}

/// Replays dead deliveries.
fn replay<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let event = match arguments.first() {
            Some(id) => Some(uuid::Uuid::parse_str(id).map_err(|_| {
                Error::validation("events.argument_invalid", vec![FieldError::new("event", "uuid")])
            })?),
            None => None,
        };
        let replayed = Events::new(context.db.clone()).replay(event).await?;
        Ok(vec![format!("{replayed} delivery(ies) queued again")])
    })
}

/// Turns a listener on or off.
fn listener<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let enabled = match argument(arguments, 2, "on-or-off")? {
            "on" => true,
            "off" => false,
            _ => {
                return Err(Error::validation(
                    "events.argument_invalid",
                    vec![FieldError::new("on-or-off", "one_of")],
                ));
            }
        };
        Events::new(context.db.clone())
            .set_listener(
                argument(arguments, 0, "module")?,
                argument(arguments, 1, "name")?,
                enabled,
            )
            .await?;
        Ok(vec![if enabled { "on" } else { "off" }.to_owned()])
    })
}

/// Shows the audit log.
fn audit<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let entries = Events::new(context.db.clone()).audit(None, None, 50).await?;
        Ok(entries
            .iter()
            .map(|e| format!("{}  {:<32} {}", e.at, e.code, e.summary))
            .collect())
    })
}
