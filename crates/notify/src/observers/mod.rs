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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What notify does when other modules change things: a grade the person may see, or a state a teacher
//! decided, becomes a notification in their inbox. Both run after the change commits, and what a person may
//! see is decided by the module that owns it.

use kampus_access::{Actor, PrincipalId};
use kampus_common::Error;
use kampus_courses::CourseId;
use kampus_module::{EventRun, JobFuture, ListenerDeclaration};
use kampus_records::{RecordId, Records};
use serde_json::json;

use crate::Notify;

/// The listeners of this module.
pub const LISTENERS: &[ListenerDeclaration] = &[
    ListenerDeclaration {
        event: kampus_records::events::GRADE_CHANGED,
        name: "notify_grade_posted",
        run: notify_grade_posted,
    },
    ListenerDeclaration {
        event: kampus_progress::events::EXCEPTION_CHANGED,
        name: "notify_state_decided",
        run: notify_state_decided,
    },
    ListenerDeclaration {
        event: kampus_progress::events::COURSE_COMPLETED,
        name: "notify_course_completed",
        run: notify_course_completed,
    },
];

/// Tells a person they completed a course.
fn notify_course_completed(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let account: PrincipalId = payload(&run, "account")?;
        if !course_notifies(&run).await? {
            return Ok(());
        }
        run.services
            .service::<Notify>()?
            .notify(
                kampus_progress::settings::notifications::COURSE_COMPLETED,
                account,
                json!({ "course": run.event.payload["course"] }),
                Some(&run.event.id.to_string()),
            )
            .await
            .map(drop)
    })
}

/// Whether the course of a progress event tells its people about their progress (`progress.notify`).
async fn course_notifies(run: &EventRun<'_>) -> Result<bool, Error> {
    let Some(scope) = run.event.scope else {
        return Ok(true);
    };
    run.services
        .service::<kampus_progress::Progress>()?
        .notifies(kampus_access::ScopeId::from_uuid(scope))
        .await
}

/// The person and the course of an event, and what else its payload names.
fn payload<T: serde::de::DeserializeOwned>(run: &EventRun<'_>, field: &str) -> Result<T, Error> {
    serde_json::from_value(run.event.payload[field].clone())
        .map_err(|_| Error::validation("notify.event_invalid", vec![]))
}

/// Tells a person about a new grade when they may see it: a grade that is hidden is not announced.
fn notify_grade_posted(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let record: RecordId = payload(&run, "record")?;
        let course: CourseId = payload(&run, "course")?;
        let account: PrincipalId = payload(&run, "account")?;
        let records = run.services.service::<Records>()?;
        let shown = records
            .grades_for(&Actor::Principal(account), course, account)
            .await
            .is_ok_and(|grades| grades.iter().any(|g| g.record == record));
        if !shown {
            return Ok(());
        }
        run.services
            .service::<Notify>()?
            .notify(
                kampus_records::settings::notifications::GRADE_POSTED,
                account,
                json!({ "record": record, "course": course }),
                Some(&run.event.id.to_string()),
            )
            .await
            .map(drop)
    })
}

/// Tells a person that a teacher decided their state on a course module.
fn notify_state_decided(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let account: PrincipalId = payload(&run, "account")?;
        if !course_notifies(&run).await? {
            return Ok(());
        }
        run.services
            .service::<Notify>()?
            .notify(
                kampus_progress::settings::notifications::STATE_DECIDED,
                account,
                json!({
                    "module": run.event.payload["module"],
                    "course": run.event.payload["course"],
                    "state": run.event.payload["state"],
                }),
                Some(&run.event.id.to_string()),
            )
            .await
            .map(drop)
    })
}
