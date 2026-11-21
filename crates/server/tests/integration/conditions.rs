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

//! Access conditions through the API: a teacher restricts course modules by the progress on another one, a
//! grade, a group and dates, a learner sees what they may open, and deleting what a condition depends on
//! asks first.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::site::{Answer, Site, site};

/// Adds a page to a section; returns its id.
async fn page(site: &Site, admin: &str, section: &str, title: &str) -> String {
    let created = site
        .call(
            Method::POST,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(admin),
            &[],
            Some(json!({ "type": "activities.page", "title": title, "settings": { "content": title } })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    created.body["id"].as_str().unwrap().to_owned()
}

/// Puts conditions on a course module.
async fn restrict(site: &Site, who: &str, module: &str, conditions: Value) -> Answer {
    site.call(
        Method::PUT,
        &format!("/api/v1/activities/course-modules/{module}/conditions"),
        Some(who),
        &[],
        Some(conditions),
    )
    .await
}

/// The course modules of a section as someone sees them, by title, with whether they may open each.
async fn listed(site: &Site, who: &str, section: &str) -> Vec<(String, bool)> {
    let answer = site
        .call(
            Method::GET,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(who),
            &[],
            None,
        )
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    answer
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["title"].as_str().unwrap().to_owned(),
                m["available"].as_bool().unwrap(),
            )
        })
        .collect()
}

/// A course with an enrolled learner and its first section: the course, the learner id, the learner session
/// and the section.
async fn course_with_learner(site: &Site, admin: &str) -> (String, String, String, String) {
    let (_, course) = site.course(admin).await;
    let (lea_id, lea) = site.learner(admin, "lea@example.org").await;
    let instance = site
        .call(
            Method::POST,
            &format!("/api/v1/enrol/courses/{course}/enrolment-instances"),
            Some(admin),
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
    let sections = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/courses/{course}/sections"),
            Some(admin),
            &[],
            None,
        )
        .await;
    let section = sections.body[0]["id"].as_str().unwrap().to_owned();
    (course, lea_id, lea, section)
}

/// A title and whether it may be opened, for comparing listings.
fn entry(title: &str, open: bool) -> (String, bool) {
    (title.to_owned(), open)
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_learner_opens_what_their_progress_and_grade_allow() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (course, lea_id, lea, section) = course_with_learner(&site, &admin).await;
    let intro = page(&site, &admin, &section, "Intro").await;
    let next = page(&site, &admin, &section, "Next").await;
    let bonus = page(&site, &admin, &section, "Bonus").await;
    site.call(
        Method::PUT,
        &format!("/api/v1/progress/course-modules/{intro}/rules"),
        Some(&admin),
        &[],
        Some(json!({ "kinds": ["view"] })),
    )
    .await;

    let set = restrict(
        &site,
        &admin,
        &next,
        json!({ "tree": { "operator": "all", "children": [
            { "operator": "leaf", "kind": "progress.state", "params": { "module": intro, "state": "done" } }
        ] } }),
    )
    .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.body);
    assert_eq!(set.body["tree"]["children"][0]["params"]["state"], "done");
    assert_eq!(set.body["show_when_unmet"], true);
    let denied = restrict(&site, &lea, &next, Value::Null).await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    let record = site
        .call(
            Method::POST,
            &format!("/api/v1/records/courses/{course}/records/items"),
            Some(&admin),
            &[],
            Some(json!({ "title": "Quiz", "max": 10 })),
        )
        .await;
    let record = record.body["id"].as_str().unwrap().to_owned();
    let hidden = restrict(
        &site,
        &admin,
        &bonus,
        json!({ "show_when_unmet": false, "tree": {
            "operator": "leaf", "kind": "records.grade", "params": { "record": record, "min": 50 }
        } }),
    )
    .await;
    assert_eq!(hidden.status, StatusCode::OK, "{}", hidden.body);

    assert_eq!(
        listed(&site, &lea, &section).await,
        [entry("Intro", true), entry("Next", false)],
        "the bonus is hidden, the next page is listed but closed"
    );
    let closed = site
        .call(
            Method::GET,
            &format!("/api/v1/activities/course-modules/{next}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(closed.body["available"], false);
    assert_eq!(closed.body["settings"], json!({}), "nothing of what it holds");
    assert_eq!(
        listed(&site, &admin, &section).await.len(),
        3,
        "teachers open everything"
    );

    site.call(
        Method::POST,
        &format!("/api/v1/progress/course-modules/{intro}/views"),
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
        Some(json!({ "grades": [{ "account": lea_id, "raw": 7 }] })),
    )
    .await;
    assert_eq!(
        listed(&site, &lea, &section).await,
        [entry("Intro", true), entry("Next", true), entry("Bonus", true)]
    );

    let refused = site
        .call(
            Method::DELETE,
            &format!("/api/v1/activities/course-modules/{intro}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.body);
    assert_eq!(refused.code(), "activities.module_required_by_conditions");
    let removed = site
        .call(
            Method::DELETE,
            &format!("/api/v1/activities/course-modules/{intro}?remove_conditions=true"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.body);
    let left = site
        .call(
            Method::GET,
            &format!("/api/v1/activities/course-modules/{next}/conditions"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(
        left.body,
        Value::Null,
        "the tree that only held that leaf is gone"
    );
    site.drop().await;
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn groups_dates_and_sections_restrict_and_wrong_trees_are_refused() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (course, lea_id, lea, section) = course_with_learner(&site, &admin).await;
    let team = page(&site, &admin, &section, "Team").await;
    let later = page(&site, &admin, &section, "Later").await;
    restrict(
        &site,
        &admin,
        &team,
        json!({ "tree": { "operator": "leaf", "kind": "activities.group", "params": {} } }),
    )
    .await;
    let dated = restrict(
        &site,
        &admin,
        &later,
        json!({ "tree": { "operator": "any", "children": [
            { "operator": "leaf", "kind": "activities.date", "params": { "opens_at": "2999-01-01T00:00:00Z" } },
            { "operator": "not", "children": [
                { "operator": "leaf", "kind": "activities.date", "params": { "closes_at": "2999-01-01T00:00:00Z" } }
            ] }
        ] } }),
    )
    .await;
    assert_eq!(dated.status, StatusCode::OK, "{}", dated.body);
    assert_eq!(
        listed(&site, &lea, &section).await,
        [entry("Team", false), entry("Later", false)]
    );

    let group = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/courses/{course}/groups"),
            Some(&admin),
            &[],
            Some(json!({ "name": "Red" })),
        )
        .await;
    let group = group.body["id"].as_str().unwrap().to_owned();
    let added = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/groups/{group}/members"),
            Some(&admin),
            &[],
            Some(json!({ "account": lea_id })),
        )
        .await;
    assert!(added.status.is_success(), "{}", added.body);
    assert_eq!(
        listed(&site, &lea, &section).await,
        [entry("Team", true), entry("Later", false)]
    );

    let spanish = page(&site, &admin, &section, "Spanish").await;
    let by_language = restrict(
        &site,
        &admin,
        &spanish,
        json!({ "tree": { "operator": "leaf", "kind": "activities.profile",
            "params": { "field": "language_code", "operator": "is", "value": "ES" } } }),
    )
    .await;
    assert_eq!(by_language.status, StatusCode::OK, "{}", by_language.body);
    let profile = "/api/v1/users/me/profile".to_owned();
    let read = site.call(Method::GET, &profile, Some(&lea), &[], None).await;
    let version = read.headers["etag"].to_str().unwrap().to_owned();
    let changed = site
        .call(
            Method::PUT,
            &profile,
            Some(&lea),
            &[("if-match", &version)],
            Some(json!({ "language_code": "es" })),
        )
        .await;
    assert!(changed.status.is_success(), "{}", changed.body);
    assert_eq!(
        listed(&site, &lea, &section).await,
        [entry("Team", true), entry("Later", false), entry("Spanish", true)]
    );

    let closed_section = site
        .call(
            Method::PUT,
            &format!("/api/v1/activities/sections/{section}/conditions"),
            Some(&admin),
            &[],
            Some(json!({ "show_when_unmet": false, "tree": {
                "operator": "leaf", "kind": "activities.date", "params": { "opens_at": "2999-01-01T00:00:00Z" }
            } })),
        )
        .await;
    assert_eq!(closed_section.status, StatusCode::OK, "{}", closed_section.body);
    assert_eq!(
        listed(&site, &lea, &section).await,
        Vec::<(String, bool)>::new(),
        "the section hides everything in it"
    );
    assert_eq!(listed(&site, &admin, &section).await.len(), 3);

    for (tree, code) in [
        (
            json!({ "operator": "leaf", "kind": "nobody.knows" }),
            "activities.condition_kind_unknown",
        ),
        (
            json!({ "operator": "not", "children": [] }),
            "activities.conditions_invalid",
        ),
        (
            json!({ "operator": "leaf", "kind": "activities.date", "params": {} }),
            "activities.condition_invalid",
        ),
        (
            json!({ "operator": "leaf", "kind": "progress.state", "params": { "module": team, "state": "maybe" } }),
            "progress.condition_invalid",
        ),
    ] {
        let refused = restrict(&site, &admin, &later, json!({ "tree": tree })).await;
        assert_eq!(
            refused.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{}",
            refused.body
        );
        assert_eq!(refused.code(), code);
    }
    let kept = site
        .call(
            Method::GET,
            &format!("/api/v1/activities/course-modules/{later}/conditions"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(
        kept.body["tree"]["operator"], "any",
        "a refused tree changes nothing"
    );
    site.drop().await;
}
