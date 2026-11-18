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

//! The `kampus-server` command: `serve` (the default), `upgrade` and `check`.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use kampus_server::{Config, run};

/// Kampus: the modular learning platform.
#[derive(Debug, Parser)]
#[command(name = "kampus-server", version)]
struct Cli {
    /// What to do; `serve` when omitted.
    #[command(subcommand)]
    command: Option<Command>,
}

/// The commands of the binary.
#[derive(Debug, Subcommand)]
enum Command {
    /// Serve the role of `KAMPUS_ROLE` until stopped.
    Serve,
    /// Apply every pending upgrade step, from any supported release.
    Upgrade {
        /// Create a new installation even if the database holds other installations.
        #[arg(long)]
        new_installation: bool,
    },
    /// Report whether the database matches this build, without changing anything.
    Check,
}

/// Prints a line for the operator, before or instead of the logs.
#[allow(
    clippy::print_stderr,
    reason = "the command reports to the operator on standard error"
)]
fn report(line: &str) {
    eprintln!("{line}");
}

#[tokio::main]
async fn main() -> ExitCode {
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
    run::init_logging(&config);

    let result = match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => run::serve(&config).await,
        Command::Upgrade { new_installation } => {
            run::upgrade(&config, new_installation).await.map(|applied| {
                report(&format!(
                    "kampus-server: {} upgrade step(s) applied",
                    applied.len()
                ));
                for step in applied {
                    report(&format!("  {step}"));
                }
            })
        }
        Command::Check => run::check(&config).await.and_then(|plan| {
            report(&format!(
                "kampus-server: installed {}, {} pending step(s)",
                plan.installed,
                plan.pending.len()
            ));
            for step in &plan.pending {
                report(&format!("  pending {step}"));
            }
            for problem in &plan.problems {
                report(&format!("  problem: {problem}"));
            }
            if plan.is_current() {
                Ok(())
            } else {
                Err(kampus_common::Error::conflict("server.schema_not_current"))
            }
        }),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = %error, "kampus-server failed");
            report(&format!("kampus-server: {error}"));
            ExitCode::FAILURE
        }
    }
}
