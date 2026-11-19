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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The command line of `kampus-server`: what each command prints and how it exits.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use super::status::{ComponentStatus, history, statuses};
use crate::Config;

/// Kampus: the modular learning platform.
#[derive(Debug, Parser)]
#[command(name = "kampus-server", version)]
pub struct Cli {
    /// What to do; `serve` when omitted.
    #[command(subcommand)]
    command: Option<Command>,
}

/// The commands of the binary.
#[derive(Debug, Subcommand)]
enum Command {
    /// Serve the role of `KAMPUS_ROLE` until stopped.
    Serve,
    /// Apply every pending upgrade step and register the components, from any supported release.
    Upgrade {
        /// Create a new installation even if the database holds other installations.
        #[arg(long)]
        new_installation: bool,
    },
    /// Report whether the database matches this build, without changing anything.
    Check,
    /// List the components of the build and of the installation, with their versions and state.
    Components {
        /// Print JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Show the latest upgrades, downgrades and uninstalls.
    History {
        /// How many runs to show.
        #[arg(long, default_value_t = 20)]
        limit: i64,
    },
    /// Take the installation back to this build's versions, undoing newer steps.
    Downgrade {
        /// Confirm: undoing steps can remove data that newer versions added.
        #[arg(long)]
        yes: bool,
    },
    /// Remove a component that is no longer in the build: its tables and its data.
    Uninstall {
        /// The component (`activity_attendance`).
        component: String,
        /// Confirm: the component's data is deleted.
        #[arg(long)]
        yes: bool,
    },
}

/// Prints a line for the operator, on standard error, before or instead of the logs.
#[allow(
    clippy::print_stderr,
    reason = "the command reports to the operator on standard error"
)]
fn report(line: &str) {
    eprintln!("{line}");
}

/// Prints the result of a command that is meant to be read or piped.
#[allow(clippy::print_stdout, reason = "listings go to standard output")]
fn print(line: &str) {
    println!("{line}");
}

/// Parses the command line, runs the command and returns how the process exits.
pub async fn run() -> ExitCode {
    let cli = Cli::parse();
    let config = match Config::from_environment() {
        Ok(config) => config,
        Err(error) => {
            report(&format!(
                "kampus-server: invalid configuration ({})",
                error.code()
            ));
            for field in error.fields() {
                report(&format!("  {}: {}", field.field, field.rule));
            }
            return ExitCode::from(2);
        }
    };
    super::init_logging(&config);
    match execute(cli.command.unwrap_or(Command::Serve), &config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = %error, "kampus-server failed");
            report(&format!("kampus-server: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// Runs one command.
async fn execute(command: Command, config: &Config) -> kampus_common::Result<()> {
    match command {
        Command::Serve => super::serve(config).await,
        Command::Upgrade { new_installation } => {
            let applied = super::upgrade(config, new_installation).await?;
            report(&format!(
                "kampus-server: {} upgrade step(s) applied",
                applied.len()
            ));
            for step in applied {
                report(&format!("  {step}"));
            }
            Ok(())
        }
        Command::Check => {
            let plan = super::check(config).await?;
            report(&format!(
                "kampus-server: installed {}, {} pending step(s), {} component version(s) to register",
                plan.installed,
                plan.pending.len(),
                plan.changes.len()
            ));
            for step in &plan.pending {
                report(&format!("  pending {step}"));
            }
            for change in &plan.changes {
                report(&format!(
                    "  {} {} -> {}",
                    change.component,
                    change.from.as_deref().unwrap_or("new"),
                    change.to.as_deref().unwrap_or("removed")
                ));
            }
            for problem in &plan.problems {
                report(&format!("  problem: {problem}"));
            }
            if plan.is_current() {
                Ok(())
            } else {
                Err(kampus_common::Error::conflict("server.schema_not_current"))
            }
        }
        Command::Components { json } => {
            let statuses = statuses(config).await?;
            if json {
                print(&serde_json::to_string_pretty(&statuses).unwrap_or_default());
            } else {
                print_statuses(&statuses);
            }
            Ok(())
        }
        Command::History { limit } => {
            for run in history(config, limit).await? {
                print(&format!(
                    "{} {} {} -> {} {}{}",
                    run.started_at,
                    run.operation,
                    run.release_from.as_deref().unwrap_or("new"),
                    run.release_to,
                    run.outcome,
                    run.error.map_or_else(String::new, |e| format!(": {e}"))
                ));
                for change in run.changes {
                    print(&format!(
                        "  {} {} -> {} ({} step(s))",
                        change.component,
                        change.from.as_deref().unwrap_or("new"),
                        change.to.as_deref().unwrap_or("removed"),
                        change.steps
                    ));
                }
            }
            Ok(())
        }
        Command::Downgrade { yes } => {
            if !yes {
                return Err(kampus_common::Error::conflict("server.confirmation_needed")
                    .with_param("command", "downgrade --yes"));
            }
            let done = super::downgrade(config).await?;
            report(&format!("kampus-server: {} step(s) undone", done.reverted.len()));
            for step in done.reverted {
                report(&format!("  {step}"));
            }
            Ok(())
        }
        Command::Uninstall { component, yes } => {
            if !yes {
                return Err(kampus_common::Error::conflict("server.confirmation_needed")
                    .with_param("command", format!("uninstall {component} --yes")));
            }
            let change = super::uninstall(config, &component).await?;
            report(&format!(
                "kampus-server: {component} {} uninstalled",
                change.from.unwrap_or_default()
            ));
            Ok(())
        }
    }
}

/// Prints the components as a table.
fn print_statuses(statuses: &[ComponentStatus]) {
    print(&format!(
        "{:<24} {:<8} {:<9} {:<10} {:<10} {:<6} {}",
        "COMPONENT", "KIND", "ORIGIN", "BUILD", "INSTALLED", "STAGE", "STATE"
    ));
    for s in statuses {
        print(&format!(
            "{:<24} {:<8} {:<9} {:<10} {:<10} {:<6} {}",
            s.name,
            s.kind,
            s.origin,
            s.build_version.as_deref().unwrap_or("-"),
            s.installed_version.as_deref().unwrap_or("-"),
            s.maturity,
            s.state
        ));
        for problem in &s.problems {
            print(&format!("    {problem}"));
        }
    }
}
