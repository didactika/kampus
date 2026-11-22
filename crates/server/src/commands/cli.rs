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
#[command(
    name = "kampus-server",
    version,
    after_help = "Each component adds its own commands: kampus-server <component> --help lists them."
)]
pub struct Cli {
    /// What to do; `serve` when omitted.
    #[command(subcommand)]
    command: Option<Command>,
}

/// The commands of the binary.
#[derive(Debug, Subcommand)]
enum Command {
    /// Serve the role of `KAMPUS_SERVER_ROLE` until stopped.
    Serve,
    /// Install Kampus: write `config.toml`, asking for what is missing, then create the installation,
    /// the first campus and its administrator; an existing installation is left as it is.
    Install {
        /// Never ask: take everything from the file and the environment.
        #[arg(long)]
        non_interactive: bool,
    },
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
    /// Copy every stored file to another store, verifying each, before switching `KAMPUS_STORAGE_URL` to it;
    /// a new run skips what was already copied.
    CopyStorage {
        /// The other store (`file:///path` or `s3://bucket/prefix`), with the options of the current one.
        url: String,
    },
    /// Generate a VAPID key for push notifications: the private key for `KAMPUS_PUSH_VAPID_PRIVATE_KEY` and the
    /// public key browsers subscribe with.
    PushKey,
    /// Print what every module of the build declares (permissions, settings, features, terms, events,
    /// notification types, jobs) as JSON, without a configuration or a database.
    Reference,
    /// A component and one of its commands: `kampus-server identity purge`.
    #[command(external_subcommand)]
    Component(Vec<String>),
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
    if let Some(Command::Install { non_interactive }) = cli.command {
        return finish(install(!non_interactive).await);
    }
    if let Some(Command::PushKey) = cli.command {
        return finish(push_key());
    }
    if let Some(Command::Reference) = cli.command {
        print(
            &serde_json::to_string_pretty(&super::reference::reference(crate::wiring::registry::MODULES))
                .unwrap_or_default(),
        );
        return ExitCode::SUCCESS;
    }
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
    finish(execute(cli.command.unwrap_or(Command::Serve), &config).await)
}

/// How the process exits after a command.
fn finish(result: kampus_common::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = %error, "kampus-server failed");
            report(&format!("kampus-server: {error}"));
            for field in error.fields() {
                report(&format!("  {}: {}", field.field, field.rule));
            }
            ExitCode::FAILURE
        }
    }
}

/// Installs, and tells the operator what happened.
async fn install(interactive: bool) -> kampus_common::Result<()> {
    let path = crate::config::file::path();
    match super::install::install(&path, &|name| std::env::var(name).ok(), interactive).await? {
        super::install::Installed::Existing { config, current } => {
            report(&format!(
                "kampus-server: the database already holds this installation; nothing was changed ({}).",
                config.display()
            ));
            if !current {
                report("  It does not match this build: run `kampus-server upgrade`.");
            }
        }
        super::install::Installed::New {
            config,
            release,
            administrator,
        } => {
            report(&format!(
                "kampus-server: Kampus {release} installed, configured in {}.",
                config.display()
            ));
            report(&format!("  Sign in as {administrator}."));
        }
    }
    Ok(())
}

/// Runs one command.
async fn execute(command: Command, config: &Config) -> kampus_common::Result<()> {
    match command {
        Command::Serve => super::serve(config).await,
        Command::Component(arguments) => component(config, &arguments).await,
        Command::Install { .. } | Command::PushKey | Command::Reference => Ok(()),
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
        Command::Check => check(config).await,
        Command::CopyStorage { url } => copy_storage(config, &url).await,
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

/// Reports whether the installation matches the build.
async fn check(config: &Config) -> kampus_common::Result<()> {
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

/// Runs a command of a component, or lists its commands.
async fn component(config: &Config, arguments: &[String]) -> kampus_common::Result<()> {
    let Some((name, rest)) = arguments.split_first() else {
        return Ok(());
    };
    let Some(module) = crate::wiring::registry::MODULES.iter().find(|m| m.name == name) else {
        return Err(
            kampus_common::Error::not_found("server.unknown_command").with_param("command", name.as_str())
        );
    };
    let Some((command_name, given)) = rest
        .split_first()
        .filter(|(c, _)| !matches!(c.as_str(), "--help" | "-h" | "help"))
    else {
        print(&format!(
            "{name} {}: {}",
            module.manifest.version, module.manifest.summary
        ));
        if module.commands.is_empty() {
            print("  No commands.");
        }
        for command in module.commands {
            print(&format!("  {:<40} {}", command.usage(name), command.summary));
        }
        return Ok(());
    };
    let Some(command) = module.commands.iter().find(|c| c.name == command_name) else {
        return Err(kampus_common::Error::not_found("server.unknown_command")
            .with_param("command", format!("{name} {command_name}"))
            .with_param("help", format!("kampus-server {name} --help")));
    };
    command.check(name, given)?;
    let db = super::connect(config).await?;
    let context = kampus_module::CommandContext {
        db: db.clone(),
        secret_key: config
            .secret_key
            .as_ref()
            .map(kampus_identity::SecretKey::to_base64),
    };
    let lines = (command.run)(&context, given).await;
    db.close().await;
    for line in lines? {
        print(&line);
    }
    Ok(())
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

/// Prints a new VAPID key and its public key, without reading the configuration.
fn push_key() -> kampus_common::Result<()> {
    let private_key = kampus_push_webpush::WebPush::generate_key()?;
    let public_key = kampus_push::Pusher::public_key(&kampus_push_webpush::WebPush::new(
        &private_key,
        "mailto:operator@example.org",
    )?);
    print(&format!("KAMPUS_PUSH_VAPID_PRIVATE_KEY={private_key}"));
    report(&format!("kampus-server: public key {public_key}"));
    Ok(())
}

/// Copies every stored file to another store, verifying each.
async fn copy_storage(config: &Config, url: &str) -> kampus_common::Result<()> {
    let db = super::connect(config).await?;
    let ports = crate::wiring::ports::Ports::open(config).await?;
    let target = kampus_storage::open(
        crate::wiring::adapters::STORAGE,
        kampus_common::AdapterSettings {
            url: url.to_owned(),
            prefix: config.database_prefix.as_str().to_owned(),
            options: config.storage_options.clone(),
        },
    )
    .await?;
    target.check().await?;
    let access = kampus_access::Access::new(db.clone(), std::sync::Arc::new(kampus_common::SystemClock));
    let files = kampus_files::Files::new(db, access, ports.storage, [0; 32], String::new());
    let copied = files.copy_to(target.as_ref()).await?;
    report(&format!(
        "kampus-server: {} file(s) copied, {} already there; switch KAMPUS_STORAGE_URL to {url} when no process writes",
        copied.copied, copied.present
    ));
    Ok(())
}
