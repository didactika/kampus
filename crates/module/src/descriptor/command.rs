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

//! The commands a component offers on the command line, run as `kampus-server <component> <command>`.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use kampus_common::Result;
use kampus_db::Database;

/// What a command returns: the lines it reports.
pub type CommandFuture<'c> = Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'c>>;

/// Where a command runs: the database of the installation and what the configuration gives it.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The database of the installation.
    pub db: Database,
    /// The secret key of the installation, in base64, for components that seal secrets.
    pub secret_key: Option<String>,
}

/// One argument of a command, positional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argument {
    /// Its name, shown in the help.
    pub name: &'static str,
    /// What it is.
    pub summary: &'static str,
    /// Whether the command needs it.
    pub required: bool,
}

/// A command of a component.
#[derive(Clone, Copy)]
pub struct Command {
    /// Its name, in kebab case (`run-deletions`).
    pub name: &'static str,
    /// What it does, in one sentence.
    pub summary: &'static str,
    /// Its arguments, in order.
    pub arguments: &'static [Argument],
    /// Runs it with the arguments given, already checked against [`Command::arguments`].
    pub run: for<'c> fn(&'c CommandContext, &'c [String]) -> CommandFuture<'c>,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}

impl Command {
    /// The usage line of the command for a component.
    #[must_use]
    pub fn usage(&self, component: &str) -> String {
        let arguments: Vec<String> = self
            .arguments
            .iter()
            .map(|a| {
                if a.required {
                    format!("<{}>", a.name)
                } else {
                    format!("[{}]", a.name)
                }
            })
            .collect();
        format!("kampus-server {component} {} {}", self.name, arguments.join(" "))
            .trim_end()
            .to_owned()
    }

    /// Checks the number of arguments given.
    ///
    /// # Errors
    ///
    /// `validation` (`module.command_arguments`) with the usage, when too few or too many are given.
    pub fn check(&self, component: &str, given: &[String]) -> Result<()> {
        let required = self.arguments.iter().filter(|a| a.required).count();
        if (required..=self.arguments.len()).contains(&given.len()) {
            Ok(())
        } else {
            Err(kampus_common::Error::validation(
                "module.command_arguments",
                vec![kampus_common::FieldError::new("arguments", "count")],
            )
            .with_param("usage", self.usage(component)))
        }
    }
}
