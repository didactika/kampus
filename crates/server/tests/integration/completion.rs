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

//! Course progress through the API: a teacher says what a course requires, the job evaluates the people the
//! course follows, a completion is recorded and revoked when a criterion stops being met, and a teacher of
//! the role a criterion names approves.

use axum::http::{Method, StatusCode};
use kampus_events::{Dispatcher, Events};
use kampus_progress::Progress;
use kampus_server::wiring::registry::MODULES;
use serde_json::{Value, json};

use crate::site::{Site, site};

/// Delivers the events to their listeners, then runs the queue of course progress until both are empty.
async fn run_queue(site: &Site) {
    let dispatcher = Dispatcher::new(Events::new(site.db.clone()), MODULES, site.context.clone());
    let progress = site.context.service::<Progress>().unwrap();
    loop {
        while dispatcher.run_once().await.unwrap() {}
        if progress.recalculate_courses().await.unwrap() == 0 {
            break;
        }
    }
}

/// The course progress of the caller.
async fn mine(site: &Site, who: &str, course: &str) -> Value {
    let answer = site
        .call(
            Method::GET,
            &format!("/api/v1/progress/courses/{course}/my-course-progress"),
            Some(who),
            &[],
            None,
        )
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    answer.body
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_learner_completes_a_course_by_its_criteria_and_loses_it_when_one_stops_holding() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let (lea_id, lea) = site.learner(&admin, "lea@example.org").await;
    let instance = site
        .call(
            Method::POST,
            &format!("/api/v1/enrol/courses/{course}/enrolment-instances"),
            Some(&admin),
            &[],
            Some(json!({ "method": "self", "name": "Self", "role": site.roles["learner"] })),
        )
        .await;
    let instance = instance.body["id"].as_str().unwrap().to_owned();
    site.call(
        Method::POST,
        &format!("/api/v1/enrol/enrolment-instances/{instance}/self"),
        Some(&lea),
        &[],
        Some(json!({})),
    )
    .await;
    let sections = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/courses/{course}/sections"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let section = sections.body[0]["id"].as_str().unwrap().to_owned();
    let page = site
        .call(
            Method::POST,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(&admin),
            &[],
            Some(json!({ "type": "activities.page", "title": "Read me", "settings": { "content": "x" } })),
        )
        .await;
    let page = page.body["id"].as_str().unwrap().to_owned();
    site.call(
        Method::PUT,
        &format!("/api/v1/progress/course-modules/{page}/rules"),
        Some(&admin),
        &[],
        Some(json!({ "kinds": ["view"] })),
    )
    .await;
    let record = site
        .call(
            Method::POST,
            &format!("/api/v1/records/courses/{course}/records/items"),
            Some(&admin),
            &[],
            Some(json!({ "title": "Exam", "max": 10 })),
        )
        .await;
    let record = record.body["id"].as_str().unwrap().to_owned();
    let criteria = format!("/api/v1/progress/courses/{course}/criteria");
    let defined = site
        .call(
            Method::PUT,
            &criteria,
            Some(&admin),
            &[],
            Some(json!({ "aggregation": "all", "criteria": [
                { "kind": "activity", "module": page, "required": "done" },
                { "kind": "grade", "record": record, "min_percent": 50 },
                { "kind": "role_approval", "role": site.roles["teacher"] },
                { "kind": "date", "on_or_after": "2000-01-01T00:00:00Z" }
            ] })),
        )
        .await;
    assert_eq!(defined.status, StatusCode::OK, "{}", defined.body);
    assert_eq!(defined.body["criteria"].as_array().unwrap().len(), 4);
    let denied = site
        .call(
            Method::PUT,
            &criteria,
            Some(&lea),
            &[],
            Some(json!({ "criteria": [] })),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let itself = site
        .call(
            Method::PUT,
            &criteria,
            Some(&admin),
            &[],
            Some(json!({ "criteria": [{ "kind": "course", "course": course }] })),
        )
        .await;
    assert_eq!(itself.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", itself.body);

    run_queue(&site).await;
    let before = mine(&site, &lea, &course).await;
    assert_eq!(before["state"], "in_progress", "{before}");
    assert_eq!(before["criteria"][3]["met"], true, "the date has passed");

    site.call(
        Method::POST,
        &format!("/api/v1/progress/course-modules/{page}/views"),
        Some(&lea),
        &[],
        None,
    )
    .await;
    site.call(
        Method::PUT,
        &format!("/api/v1/records/records/{record}/grades"),
        Some(&admin),
        &[],
        Some(json!({ "grades": [{ "account": lea_id, "raw": 8 }] })),
    )
    .await;
    let approval = before["criteria"][2]["id"].as_str().unwrap().to_owned();
    let not_a_teacher = site
        .call(
            Method::PUT,
            &format!("/api/v1/progress/criteria/{approval}/approvals/{lea_id}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(
        not_a_teacher.status,
        StatusCode::FORBIDDEN,
        "{}",
        not_a_teacher.body
    );
    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&admin), &[], None)
        .await;
    let admin_id = me.body["id"].as_str().unwrap().to_owned();
    site.enrol(&admin, &course, &admin_id, "teacher").await;
    let approved = site
        .call(
            Method::PUT,
            &format!("/api/v1/progress/criteria/{approval}/approvals/{lea_id}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(approved.status, StatusCode::NO_CONTENT, "{}", approved.body);
    run_queue(&site).await;
    let done = mine(&site, &lea, &course).await;
    assert_eq!(done["state"], "completed", "{done}");
    assert!(done["completed_at"].is_string());
    let dispatcher = Dispatcher::new(Events::new(site.db.clone()), MODULES, site.context.clone());
    while dispatcher.run_once().await.unwrap() {}
    let inbox = site
        .call(Method::GET, "/api/v1/notify/inbox", Some(&lea), &[], None)
        .await;
    assert!(
        inbox.body.to_string().contains("progress.course_completed"),
        "{}",
        inbox.body
    );

    let report = site
        .call(
            Method::GET,
            &format!("/api/v1/progress/courses/{course}/course-progress"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(report.status, StatusCode::OK, "{}", report.body);
    assert_eq!(
        report.body.as_array().unwrap().len(),
        1,
        "only who the course follows"
    );

    site.call(
        Method::PUT,
        &format!("/api/v1/records/records/{record}/grades"),
        Some(&admin),
        &[],
        Some(json!({ "grades": [{ "account": lea_id, "raw": 2 }] })),
    )
    .await;
    run_queue(&site).await;
    assert_eq!(
        mine(&site, &lea, &course).await["state"],
        "in_progress",
        "revoked when the grade no longer holds"
    );

    let kept = defined.body["criteria"].clone();
    let mut criteria_final = json!({ "aggregation": "all", "final": true, "criteria": kept });
    criteria_final["criteria"].as_array_mut().unwrap().remove(1);
    let redefined = site
        .call(Method::PUT, &criteria, Some(&admin), &[], Some(criteria_final))
        .await;
    assert_eq!(redefined.status, StatusCode::OK, "{}", redefined.body);
    run_queue(&site).await;
    let again = mine(&site, &lea, &course).await;
    assert_eq!(
        again["state"], "completed",
        "the approval stayed with its criterion: {again}"
    );
    site.drop().await;
}
