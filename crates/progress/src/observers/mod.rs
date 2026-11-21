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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What progress does when other modules change things: a grade that changed, or was overridden, re-evaluates
//! the person on the course module it belongs to; a change of activity progress, of a grade or of a total
//! queues the person in their course, and a course completed or revoked queues them in the courses that
//! require it.

use kampus_common::Error;
use kampus_module::{EventRun, JobFuture, ListenerDeclaration};

use crate::Progress;

/// The listeners of this module.
pub const LISTENERS: &[ListenerDeclaration] = &[
    ListenerDeclaration {
        event: kampus_records::events::GRADE_CHANGED,
        name: "evaluate_after_grade",
        run: evaluate_after_grade,
    },
    ListenerDeclaration {
        event: kampus_records::events::OVERRIDDEN,
        name: "evaluate_after_override",
        run: evaluate_after_grade,
    },
    ListenerDeclaration {
        event: crate::events::ACTIVITY_CHANGED,
        name: "queue_course_after_activity",
        run: queue_course,
    },
    ListenerDeclaration {
        event: kampus_records::events::GRADE_CHANGED,
        name: "queue_course_after_grade",
        run: queue_course,
    },
    ListenerDeclaration {
        event: kampus_records::events::OVERRIDDEN,
        name: "queue_course_after_override",
        run: queue_course,
    },
    ListenerDeclaration {
        event: kampus_records::events::TOTAL_CHANGED,
        name: "queue_course_after_total",
        run: queue_course,
    },
    ListenerDeclaration {
        event: crate::events::COURSE_COMPLETED,
        name: "queue_requiring_after_completion",
        run: queue_requiring,
    },
    ListenerDeclaration {
        event: crate::events::COURSE_REVOKED,
        name: "queue_requiring_after_revocation",
        run: queue_requiring,
    },
];

/// The course and the person an event is about.
fn course_and_person(
    run: &EventRun<'_>,
) -> Result<(kampus_courses::CourseId, kampus_access::PrincipalId), Error> {
    let invalid = || Error::validation("progress.event_invalid", vec![]);
    let course = serde_json::from_value(run.event.payload["course"].clone()).map_err(|_| invalid())?;
    let account = serde_json::from_value(run.event.payload["account"].clone()).map_err(|_| invalid())?;
    Ok((course, account))
}

/// Queues the person of an event in its course, so their course progress is evaluated again.
fn queue_course(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        if run
            .event
            .payload
            .get("account")
            .is_none_or(serde_json::Value::is_null)
        {
            return Ok(());
        }
        let (course, account) = course_and_person(&run)?;
        run.services
            .service::<Progress>()?
            .queue_course(course, account)
            .await
    })
}

/// Queues the person in the courses that require the course of the event.
fn queue_requiring(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let (course, account) = course_and_person(&run)?;
        run.services
            .service::<Progress>()?
            .queue_requiring(course, account)
            .await
    })
}

/// Re-evaluates the person of a grade event on the course module of the record.
fn evaluate_after_grade(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let invalid = || Error::validation("progress.event_invalid", vec![]);
        let record: kampus_records::RecordId =
            serde_json::from_value(run.event.payload["record"].clone()).map_err(|_| invalid())?;
        let account: kampus_access::PrincipalId =
            serde_json::from_value(run.event.payload["account"].clone()).map_err(|_| invalid())?;
        let records = run.services.service::<kampus_records::Records>()?;
        let Some(module) = records.module_of_record(record).await? else {
            return Ok(());
        };
        run.services
            .service::<Progress>()?
            .reevaluate(module, account, "grade")
            .await
            .map(drop)
    })
}
