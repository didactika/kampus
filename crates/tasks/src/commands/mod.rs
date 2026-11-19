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
// @package   kampus-tasks
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The commands tasks offers on the command line, `kampus-server tasks <command>`: the operator's view of
//! jobs and schedules until their API arrives.

use kampus_common::{Error, FieldError, Result};
use kampus_module::{Argument, Command, CommandContext, CommandFuture};

use crate::{JobId, JobState, Tasks};

/// The commands of tasks.
pub const COMMANDS: &[Command] = &[
    Command {
        name: "jobs",
        summary: "List the latest jobs, optionally in one state (queued, running, succeeded, failed, cancelled)",
        arguments: &[Argument {
            name: "state",
            summary: "Only jobs in this state",
            required: false,
        }],
        run: jobs,
    },
    Command {
        name: "job",
        summary: "Show a job with its attempts",
        arguments: &[Argument {
            name: "id",
            summary: "The job",
            required: true,
        }],
        run: job,
    },
    Command {
        name: "retry",
        summary: "Queue a failed job again",
        arguments: &[Argument {
            name: "id",
            summary: "The job",
            required: true,
        }],
        run: retry,
    },
    Command {
        name: "cancel",
        summary: "Cancel a job; a running one stops at its next step",
        arguments: &[Argument {
            name: "id",
            summary: "The job",
            required: true,
        }],
        run: cancel,
    },
    Command {
        name: "schedules",
        summary: "List the schedules with their rules and next runs",
        arguments: &[],
        run: schedules,
    },
    Command {
        name: "schedule",
        summary: "Change the rule of a schedule, or turn it on or off",
        arguments: &[
            Argument {
                name: "job",
                summary: "The job type of the schedule",
                required: true,
            },
            Argument {
                name: "cron",
                summary: "Its rule, in quotes: minute hour day month weekday",
                required: true,
            },
            Argument {
                name: "time-zone",
                summary: "The time zone of the rule",
                required: true,
            },
            Argument {
                name: "on-or-off",
                summary: "on (the default) or off",
                required: false,
            },
        ],
        run: schedule,
    },
    Command {
        name: "run",
        summary: "Run a scheduled job now, outside its rule",
        arguments: &[Argument {
            name: "job",
            summary: "The job type",
            required: true,
        }],
        run: run_now,
    },
];

/// The argument at a position.
fn argument<'a>(arguments: &'a [String], position: usize, name: &'static str) -> Result<&'a str> {
    arguments
        .get(position)
        .map(String::as_str)
        .ok_or_else(|| Error::validation("tasks.argument_missing", vec![FieldError::new(name, "required")]))
}

/// A job id from the arguments.
fn job_id(arguments: &[String]) -> Result<JobId> {
    argument(arguments, 0, "id")?.parse()
}

/// Lists jobs.
fn jobs<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let state = arguments.first().map(|s| JobState::read(s));
        let jobs = Tasks::new(context.db.clone()).jobs(None, state, 50).await?;
        Ok(jobs
            .iter()
            .map(|j| {
                format!(
                    "{}  {:<10} {:<28} attempts {}  {}",
                    j.id,
                    j.state.as_str(),
                    j.type_code,
                    j.attempts,
                    j.last_error.as_deref().unwrap_or("")
                )
            })
            .collect())
    })
}

/// Shows a job.
fn job<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let (job, attempts) = Tasks::new(context.db.clone()).job(job_id(arguments)?).await?;
        let mut lines = vec![
            format!("{} {} ({})", job.id, job.type_code, job.state.as_str()),
            format!("payload {}", job.payload),
            format!("progress {}%", job.progress),
        ];
        lines.extend(attempts.iter().map(|a| {
            format!(
                "  {} {} {}",
                a.worker,
                a.outcome.as_deref().unwrap_or("running"),
                a.error.as_deref().unwrap_or("")
            )
        }));
        Ok(lines)
    })
}

/// Retries a job.
fn retry<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        Tasks::new(context.db.clone()).retry(job_id(arguments)?).await?;
        Ok(vec!["queued again".to_owned()])
    })
}

/// Cancels a job.
fn cancel<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let state = Tasks::new(context.db.clone()).cancel(job_id(arguments)?).await?;
        Ok(vec![if state == JobState::Cancelled {
            "cancelled".to_owned()
        } else {
            "asked to stop at its next step".to_owned()
        }])
    })
}

/// Lists schedules.
fn schedules<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let schedules = Tasks::new(context.db.clone()).schedules().await?;
        Ok(schedules
            .iter()
            .map(|s| {
                format!(
                    "{:<28} {:<16} {:<14} {}  next {}",
                    s.type_code,
                    s.cron,
                    s.time_zone,
                    if s.enabled { "on " } else { "off" },
                    s.next_run_at
                )
            })
            .collect())
    })
}

/// Changes a schedule.
fn schedule<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let enabled = match arguments.get(3).map(String::as_str) {
            None | Some("on") => true,
            Some("off") => false,
            Some(_) => {
                return Err(Error::validation(
                    "tasks.argument_invalid",
                    vec![FieldError::new("on-or-off", "one_of")],
                ));
            }
        };
        let changed = Tasks::new(context.db.clone())
            .change_schedule(
                argument(arguments, 0, "job")?,
                argument(arguments, 1, "cron")?,
                argument(arguments, 2, "time-zone")?,
                enabled,
                "operator",
            )
            .await?;
        Ok(vec![format!(
            "{} next runs at {}",
            changed.type_code, changed.next_run_at
        )])
    })
}

/// Runs a scheduled job now.
fn run_now<'c>(context: &'c CommandContext, arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let id = Tasks::new(context.db.clone())
            .run_now(argument(arguments, 0, "job")?)
            .await?;
        Ok(vec![format!("job {id} queued")])
    })
}
