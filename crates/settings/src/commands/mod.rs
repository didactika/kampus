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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The commands settings offers on the command line, `kampus-server settings <command>`: the operator reads
//! and sets settings of the platform or of a scope, until their API arrives.

use std::sync::Arc;

use kampus_access::{Access, ScopeId};
use kampus_common::{Error, FieldError, Result, SystemClock};
use kampus_module::{Argument, Command, CommandContext, CommandFuture};
use serde_json::Value;

use crate::{Origin, Settings};

/// The argument that names a scope: its id, or `platform`.
const SCOPE: Argument = Argument {
    name: "scope",
    summary: "The id of a scope, or platform",
    required: true,
};

/// The argument that names a setting.
const KEY: Argument = Argument {
    name: "key",
    summary: "The key of the setting, such as identity.self_sign_up",
    required: true,
};

/// The commands of settings.
pub const COMMANDS: &[Command] = &[
    Command {
        name: "list",
        summary: "List the declared settings with their defaults and levels",
        arguments: &[],
        run: list,
    },
    Command {
        name: "get",
        summary: "Show the value of a setting in a scope and where it comes from",
        arguments: &[KEY, SCOPE],
        run: get,
    },
    Command {
        name: "set",
        summary: "Set a setting in a scope, with a JSON value (true, 12, \"text\"), optionally locked",
        arguments: &[
            KEY,
            SCOPE,
            Argument {
                name: "value",
                summary: "The value, as JSON",
                required: true,
            },
            Argument {
                name: "locked",
                summary: "locked, so no scope below may change it",
                required: false,
            },
        ],
        run: set,
    },
    Command {
        name: "unset",
        summary: "Remove the value of a setting in a scope, so it comes from above again",
        arguments: &[KEY, SCOPE],
        run: unset,
    },
    Command {
        name: "history",
        summary: "Show the changes of a setting in a scope",
        arguments: &[KEY, SCOPE],
        run: history,
    },
];

/// The settings of the installation, with access.
fn settings(context: &CommandContext) -> (Settings, Access) {
    let access = Access::new(context.db.clone(), Arc::new(SystemClock));
    (Settings::new(context.db.clone(), access.clone()), access)
}

/// The argument at a position.
fn argument<'a>(arguments: &'a [String], position: usize, name: &'static str) -> Result<&'a str> {
    arguments.get(position).map(String::as_str).ok_or_else(|| {
        Error::validation(
            "settings.argument_missing",
            vec![FieldError::new(name, "required")],
        )
    })
}

/// The scope an argument names.
async fn scope(access: &Access, text: &str) -> Result<ScopeId> {
    if text == "platform" {
        return access
            .platform()
            .await?
            .ok_or_else(|| Error::not_found("settings.platform_missing"));
    }
    text.parse()
}

/// Lists the definitions.
fn list<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let (settings, _) = settings(context);
        Ok(settings
            .definitions()
            .await?
            .iter()
            .map(|d| {
                format!(
                    "{:<40} default {:<10} levels {:<24} {}",
                    d.key,
                    d.default_value,
                    d.levels.join(","),
                    d.summary
                )
            })
            .collect())
    })
}

/// Shows a resolved value.
fn get<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let (settings, access) = settings(context);
        let scope = scope(&access, argument(arguments, 1, "scope")?).await?;
        let resolved = settings.resolve(argument(arguments, 0, "key")?, scope).await?;
        let origin = match resolved.origin {
            Origin::Default => "the default".to_owned(),
            Origin::Scope { scope, locked: true } => format!("scope {scope}, locked"),
            Origin::Scope { scope, locked: false } => format!("scope {scope}"),
        };
        Ok(vec![format!(
            "{} = {} (from {origin})",
            resolved.key, resolved.value
        )])
    })
}

/// Sets a value.
fn set<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let (settings, access) = settings(context);
        let key = argument(arguments, 0, "key")?;
        let scope = scope(&access, argument(arguments, 1, "scope")?).await?;
        let value: Value = serde_json::from_str(argument(arguments, 2, "value")?).map_err(|_| {
            Error::validation("settings.value_invalid", vec![FieldError::new("value", "json")])
        })?;
        let locked = match arguments.get(3).map(String::as_str) {
            None => false,
            Some("locked") => true,
            Some(_) => {
                return Err(Error::validation(
                    "settings.argument_invalid",
                    vec![FieldError::new("locked", "one_of")],
                ));
            }
        };
        settings.set_as_operator(key, scope, Some(value), locked).await?;
        Ok(vec![format!("{key} set")])
    })
}

/// Removes a value.
fn unset<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let (settings, access) = settings(context);
        let key = argument(arguments, 0, "key")?;
        let scope = scope(&access, argument(arguments, 1, "scope")?).await?;
        settings.set_as_operator(key, scope, None, false).await?;
        Ok(vec![format!("{key} unset")])
    })
}

/// Shows the changes of a value.
fn history<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let (settings, access) = settings(context);
        let key = argument(arguments, 0, "key")?;
        let scope = scope(&access, argument(arguments, 1, "scope")?).await?;
        Ok(settings
            .history(key, scope)
            .await?
            .iter()
            .map(|c| {
                format!(
                    "{}  {} -> {}{}",
                    c.at,
                    c.before
                        .as_ref()
                        .map_or_else(|| "unset".to_owned(), ToString::to_string),
                    c.after
                        .as_ref()
                        .map_or_else(|| "unset".to_owned(), ToString::to_string),
                    if c.locked { " (locked)" } else { "" }
                )
            })
            .collect())
    })
}
