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

//! Records through the server, as a client sees them: a teacher defines a category and a record and sets
//! grades, a learner reads only their own, and overrides and locks are checked.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_teacher_grades_and_a_learner_reads_only_their_own() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let (lea_id, lea) = site.learner(&admin, "lea@example.org").await;
    let (max_id, _) = site.learner(&admin, "max@example.org").await;
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
    let joined = site
        .call(
            Method::POST,
            &format!("/api/v1/enrol/enrolment-instances/{instance}/self"),
            Some(&lea),
            &[],
            Some(json!({})),
        )
        .await;
    assert_eq!(joined.status, StatusCode::CREATED, "{}", joined.body);

    let base = format!("/api/v1/records/courses/{course}");
    let category = site
        .call(
            Method::POST,
            &format!("{base}/records/categories"),
            Some(&admin),
            &[],
            Some(json!({ "name": "Exams", "method": "natural" })),
        )
        .await;
    assert_eq!(category.status, StatusCode::CREATED, "{}", category.body);
    let category = category.body["id"].as_str().unwrap().to_owned();
    let record = site
        .call(
            Method::POST,
            &format!("{base}/records/items"),
            Some(&admin),
            &[],
            Some(json!({ "title": "Midterm", "category": category, "max": 10 })),
        )
        .await;
    assert_eq!(record.status, StatusCode::CREATED, "{}", record.body);
    let record = record.body["id"].as_str().unwrap().to_owned();
    let denied = site
        .call(
            Method::POST,
            &format!("{base}/records/items"),
            Some(&lea),
            &[],
            Some(json!({ "title": "Mine", "max": 10 })),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    let grades = format!("/api/v1/records/records/{record}/grades");
    let set = site
        .call(
            Method::PUT,
            &grades,
            Some(&admin),
            &[],
            Some(json!({ "grades": [{ "account": lea_id, "raw": 7 }, { "account": max_id, "raw": 4 }] })),
        )
        .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.body);
    assert_eq!(set.body[0]["changed"], true);
    let again = site
        .call(
            Method::PUT,
            &grades,
            Some(&admin),
            &[],
            Some(json!({ "grades": [{ "account": lea_id, "raw": 7 }] })),
        )
        .await;
    assert_eq!(again.body[0]["changed"], false);
    let out_of_range = site
        .call(
            Method::PUT,
            &grades,
            Some(&admin),
            &[],
            Some(json!({ "grades": [{ "account": lea_id, "raw": 11 }] })),
        )
        .await;
    assert_eq!(out_of_range.status, StatusCode::UNPROCESSABLE_ENTITY);
    let by_learner = site
        .call(
            Method::PUT,
            &grades,
            Some(&lea),
            &[],
            Some(json!({ "grades": [{ "account": lea_id, "raw": 10 }] })),
        )
        .await;
    assert_eq!(by_learner.status, StatusCode::FORBIDDEN);

    let mine = site
        .call(Method::GET, &format!("{base}/my-grades"), Some(&lea), &[], None)
        .await;
    assert_eq!(mine.status, StatusCode::OK, "{}", mine.body);
    assert_eq!(mine.body.as_array().unwrap().len(), 1);
    assert_eq!(mine.body[0]["final"], "7");
    let others = site
        .call(Method::GET, &format!("{grades}/{max_id}"), Some(&lea), &[], None)
        .await;
    assert_eq!(others.status, StatusCode::NOT_FOUND);
    let report = site
        .call(
            Method::GET,
            &format!("{base}/grader-report?limit=10"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(report.status, StatusCode::OK, "{}", report.body);
    assert_eq!(
        report.body["rows"].as_array().unwrap().len(),
        1,
        "max has a grade but does not take part"
    );
    let report_denied = site
        .call(
            Method::GET,
            &format!("{base}/grader-report"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(report_denied.status, StatusCode::FORBIDDEN);

    let overridden = site
        .call(
            Method::PUT,
            &format!("{grades}/{lea_id}/override"),
            Some(&admin),
            &[],
            Some(json!({ "value": 9, "feedback": "regraded" })),
        )
        .await;
    assert_eq!(overridden.status, StatusCode::OK, "{}", overridden.body);
    assert_eq!(overridden.body["grade"]["final"], "9");
    let locked = site
        .call(
            Method::PUT,
            &format!("/api/v1/records/records/{record}/lock"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(locked.status, StatusCode::OK, "{}", locked.body);
    let blocked = site
        .call(
            Method::PUT,
            &grades,
            Some(&admin),
            &[],
            Some(json!({ "grades": [{ "account": max_id, "raw": 5 }] })),
        )
        .await;
    assert_eq!(blocked.status, StatusCode::CONFLICT);
    assert_eq!(blocked.code(), "records.locked");
    let detail = site
        .call(Method::GET, &format!("{grades}/{lea_id}"), Some(&lea), &[], None)
        .await;
    assert_eq!(
        detail.body["entries"].as_array().unwrap().len(),
        2,
        "{}",
        detail.body
    );
    site.drop().await;
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn grades_survive_the_end_of_a_participation_and_come_back_with_it() {
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
    let join = format!("/api/v1/enrol/enrolment-instances/{instance}/self");
    let joined = site
        .call(Method::POST, &join, Some(&lea), &[], Some(json!({})))
        .await;
    assert_eq!(joined.status, StatusCode::CREATED, "{}", joined.body);
    let enrolment = joined.body["enrolment"].as_str().unwrap().to_owned();
    let base = format!("/api/v1/records/courses/{course}");
    let record = site
        .call(
            Method::POST,
            &format!("{base}/records/items"),
            Some(&admin),
            &[],
            Some(json!({ "title": "Quiz", "max": 10 })),
        )
        .await;
    let record = record.body["id"].as_str().unwrap().to_owned();
    site.call(
        Method::PUT,
        &format!("/api/v1/records/records/{record}/grades"),
        Some(&admin),
        &[],
        Some(json!({ "grades": [{ "account": lea_id, "raw": 7 }] })),
    )
    .await;
    let rows = |answer: &crate::site::Answer| answer.body["rows"].as_array().unwrap().clone();
    let report = site
        .call(
            Method::GET,
            &format!("{base}/grader-report"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(rows(&report).len(), 1);
    assert_eq!(rows(&report)[0]["active"], true);

    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/enrol/enrolments/{enrolment}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let version = read.headers["etag"].to_str().unwrap().to_owned();
    let removed = site
        .call(
            Method::DELETE,
            &format!("/api/v1/enrol/enrolments/{enrolment}"),
            Some(&admin),
            &[("if-match", &version)],
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.body);
    let report = site
        .call(
            Method::GET,
            &format!("{base}/grader-report"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(
        rows(&report).len(),
        0,
        "out of the report while they do not take part"
    );
    let all = site
        .call(
            Method::GET,
            &format!("{base}/grader-report?include_inactive=true"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(rows(&all)[0]["active"], false);
    assert_eq!(rows(&all)[0]["grades"][0]["final"], "7", "the grade stays");

    let back = site
        .call(Method::POST, &join, Some(&lea), &[], Some(json!({})))
        .await;
    assert_eq!(back.status, StatusCode::CREATED, "{}", back.body);
    let report = site
        .call(
            Method::GET,
            &format!("{base}/grader-report"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(rows(&report)[0]["active"], true, "back in the report on return");
    let mine = site
        .call(Method::GET, &format!("{base}/my-grades"), Some(&lea), &[], None)
        .await;
    assert_eq!(mine.body[0]["final"], "7");
    site.drop().await;
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_teacher_exports_the_grades_of_a_course_as_a_file_only_they_read() {
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
    let read = site
        .call(Method::GET, "/api/v1/users/me/profile", Some(&lea), &[], None)
        .await;
    let version = read.headers["etag"].to_str().unwrap().to_owned();
    site.call(
        Method::PUT,
        "/api/v1/users/me/profile",
        Some(&lea),
        &[("if-match", &version)],
        Some(json!({ "display_name": "=Lea, the \"first\"" })),
    )
    .await;
    let base = format!("/api/v1/records/courses/{course}");
    let record = site
        .call(
            Method::POST,
            &format!("{base}/records/items"),
            Some(&admin),
            &[],
            Some(json!({ "title": "Quiz", "max": 10 })),
        )
        .await;
    let record = record.body["id"].as_str().unwrap().to_owned();
    site.call(
        Method::PUT,
        &format!("/api/v1/records/records/{record}/grades"),
        Some(&admin),
        &[],
        Some(json!({ "grades": [{ "account": lea_id, "raw": 7.5 }] })),
    )
    .await;

    let denied = site
        .call(
            Method::POST,
            &format!("{base}/exports"),
            Some(&lea),
            &[],
            Some(json!({})),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let asked = site
        .call(
            Method::POST,
            &format!("{base}/exports"),
            Some(&admin),
            &[],
            Some(json!({})),
        )
        .await;
    assert_eq!(asked.status, StatusCode::ACCEPTED, "{}", asked.body);
    assert_eq!(asked.body["state"], "pending");
    let export = asked.body["id"].as_str().unwrap().to_owned();

    let records = site.context.service::<kampus_records::Records>().unwrap();
    assert_eq!(records.run_exports(&site.context).await.unwrap(), 1);
    let done = site
        .call(
            Method::GET,
            &format!("/api/v1/records/exports/{export}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(done.body["state"], "completed", "{}", done.body);
    let file = done.body["file"].as_str().unwrap().to_owned();
    let others = site
        .call(
            Method::GET,
            &format!("/api/v1/records/exports/{export}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(others.status, StatusCode::NOT_FOUND);
    let lea_reads = site
        .call(
            Method::GET,
            &format!("/api/v1/files/files/{file}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(lea_reads.status, StatusCode::NOT_FOUND, "{}", lea_reads.body);
    let admin_reads = site
        .call(
            Method::GET,
            &format!("/api/v1/files/files/{file}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(admin_reads.status, StatusCode::OK, "{}", admin_reads.body);

    let files = site.context.service::<kampus_files::Files>().unwrap();
    let content = files
        .read_content(kampus_files::FileId::from_uuid(file.parse().unwrap()))
        .await
        .unwrap();
    let csv = String::from_utf8(content.to_vec()).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert!(lines[0].starts_with("account,name,"), "{csv}");
    assert!(lines[0].contains("Quiz"), "{csv}");
    assert!(
        lines[1].starts_with(&format!("{lea_id},\"'=Lea, the \"\"first\"\"\",")),
        "quoted, and never read as a formula: {csv}"
    );
    assert!(lines[1].contains(",7.5"), "{csv}");

    assert_eq!(records.expire_exports(&site.context).await.unwrap(), 0, "not yet");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "update {}.\"export\" set expires_at = now() - interval '1 minute'",
        site.db.prefix().schema("records")
    )))
    .execute(site.db.pool())
    .await
    .unwrap();
    assert_eq!(records.expire_exports(&site.context).await.unwrap(), 1);
    let gone = site
        .call(
            Method::GET,
            &format!("/api/v1/records/exports/{export}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(gone.body["state"], "expired");
    assert_eq!(gone.body["file"], serde_json::Value::Null);
    site.drop().await;
}
