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

//! Customisation through the server: a campus renames the words of courses, adds a custom field that
//! courses fill in, and turns groups off in one course.

use axum::http::{Method, StatusCode};
use kampus_events::{Dispatcher, Events};
use kampus_server::wiring::registry::MODULES;
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn a_campus_renames_terms_adds_course_fields_and_turns_groups_off() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let campus = site.campus.to_string();
    let renamed = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{campus}/terms/courses.course"),
            Some(&admin),
            &[],
            Some(json!({ "language": "es", "singular": "asignatura", "plural": "asignaturas" })),
        )
        .await;
    assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.body);
    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let scope = read.body["scope"].as_str().unwrap().to_owned();
    let term = site
        .call(
            Method::GET,
            &format!("/api/v1/scopes/{scope}/terms/courses.course?language=es"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(term.body["plural"], "asignaturas", "{}", term.body);

    let field = site
        .call(
            Method::POST,
            &format!("/api/v1/scopes/{campus}/fields"),
            Some(&admin),
            &[],
            Some(
                json!({ "kind": "courses.course", "code": "credits", "name": "Credits", "type": "integer" }),
            ),
        )
        .await;
    assert_eq!(field.status, StatusCode::CREATED, "{}", field.body);
    let set = site
        .call(
            Method::PUT,
            &format!("/api/v1/courses/{course}/fields/credits"),
            Some(&admin),
            &[],
            Some(json!({ "value": 6 })),
        )
        .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.body);
    let fields = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}/fields"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(fields.body[0]["value"], 6, "{}", fields.body);

    let off = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{scope}/flags/courses.groups"),
            Some(&admin),
            &[],
            Some(json!({ "enabled": false })),
        )
        .await;
    assert_eq!(off.status, StatusCode::OK, "{}", off.body);
    let group = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/{course}/groups"),
            Some(&admin),
            &[],
            Some(json!({ "name": "Red" })),
        )
        .await;
    assert_eq!(group.status, StatusCode::CONFLICT, "{}", group.body);
    assert_eq!(group.code(), "customise.feature_off");
    site.drop().await;
}

#[tokio::test]
async fn a_course_page_is_arranged_and_the_theme_reaches_the_client_as_css() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let scope = read.body["scope"].as_str().unwrap().to_owned();
    let layout = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{scope}/layouts/courses.course"),
            Some(&admin),
            &[],
            Some(json!({ "placements": [
                { "region": "main", "widget": "courses.outline" },
                { "region": "side", "widget": "customise.text", "settings": { "heading": "Office hours" } }
            ] })),
        )
        .await;
    assert_eq!(layout.status, StatusCode::OK, "{}", layout.body);
    assert_eq!(layout.body["placements"][0]["settings"]["collapsed"], false);
    assert_eq!(layout.body["placements"][1]["settings"]["text"], "");

    let campus = site.campus.to_string();
    let menu = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{campus}/menus/customise.main"),
            Some(&admin),
            &[],
            Some(json!({ "items": [{ "label": "Courses", "target": "/courses" }] })),
        )
        .await;
    assert_eq!(menu.status, StatusCode::OK, "{}", menu.body);

    let theme = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{campus}/theme"),
            Some(&admin),
            &[],
            Some(json!({ "tokens": { "primary": "#0f766e" } })),
        )
        .await;
    assert_eq!(theme.status, StatusCode::OK, "{}", theme.body);
    let css = site
        .call(
            Method::GET,
            &format!("/api/v1/scopes/{scope}/theme.css"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert!(
        css.body.as_str().unwrap().contains("--kampus-primary: #0f766e;"),
        "{}",
        css.body
    );
    let refused = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{campus}/theme"),
            Some(&admin),
            &[],
            Some(json!({ "tokens": { "primary": "#0f766e", "on_primary": "#134e4a" } })),
        )
        .await;
    assert_eq!(
        refused.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        refused.body
    );
    assert_eq!(refused.code(), "customise.theme_contrast");
    site.drop().await;
}

#[tokio::test]
async fn a_rule_adds_who_enrols_to_a_group() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let scope = read.body["scope"].as_str().unwrap().to_owned();
    let group = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/{course}/groups"),
            Some(&admin),
            &[],
            Some(json!({ "name": "Newcomers" })),
        )
        .await;
    assert_eq!(group.status, StatusCode::CREATED, "{}", group.body);
    let group = group.body["id"].as_str().unwrap().to_owned();
    let rule = site
        .call(
            Method::POST,
            &format!("/api/v1/scopes/{scope}/rules"),
            Some(&admin),
            &[],
            Some(json!({
                "name": "Newcomers join their group",
                "trigger": "enrol.enrolment_created",
                "actions": [{ "action": "courses.add_to_group", "params": { "group": group } }],
            })),
        )
        .await;
    assert_eq!(rule.status, StatusCode::CREATED, "{}", rule.body);
    let rule = rule.body["id"].as_str().unwrap().to_owned();
    let instance = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/{course}/enrolment-instances"),
            Some(&admin),
            &[],
            Some(json!({ "method": "self", "name": "Self", "role": site.roles["learner"] })),
        )
        .await;
    assert_eq!(instance.status, StatusCode::CREATED, "{}", instance.body);
    let instance = instance.body["id"].as_str().unwrap().to_owned();
    let (lea, session) = site.learner(&admin, "lea@example.org").await;
    let joined = site
        .call(
            Method::POST,
            &format!("/api/v1/enrolment-instances/{instance}/self"),
            Some(&session),
            &[],
            Some(json!({})),
        )
        .await;
    assert_eq!(joined.status, StatusCode::CREATED, "{}", joined.body);

    let dispatcher = Dispatcher::new(Events::new(site.db.clone()), MODULES, site.context.clone());
    while dispatcher.run_once().await.unwrap() {}
    let members = site
        .call(
            Method::GET,
            &format!("/api/v1/groups/{group}/members"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert!(members.body.to_string().contains(&lea), "{}", members.body);
    let runs = site
        .call(
            Method::GET,
            &format!("/api/v1/rules/{rule}/runs"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(runs.body[0]["outcome"], "succeeded", "{}", runs.body);
    site.drop().await;
}

#[tokio::test]
async fn a_campus_exports_its_customisation_and_a_course_imports_it_after_review() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let scope = read.body["scope"].as_str().unwrap().to_owned();
    let campus = site.campus.to_string();
    site.call(
        Method::PUT,
        &format!("/api/v1/scopes/{campus}/terms/courses.section"),
        Some(&admin),
        &[],
        Some(json!({ "singular": "week", "plural": "weeks" })),
    )
    .await;
    let exported = site
        .call(
            Method::POST,
            &format!("/api/v1/scopes/{campus}/customisation-packages"),
            Some(&admin),
            &[],
            Some(json!({ "kinds": ["terms", "theme"] })),
        )
        .await;
    assert_eq!(exported.status, StatusCode::OK, "{}", exported.body);
    assert_eq!(exported.body["package"]["terms"][0]["singular"], "week");
    let preview = site
        .call(
            Method::POST,
            &format!("/api/v1/scopes/{scope}/customisation-imports"),
            Some(&admin),
            &[],
            Some(exported.body["package"].clone()),
        )
        .await;
    assert_eq!(preview.status, StatusCode::CREATED, "{}", preview.body);
    assert_eq!(preview.body["differences"][0]["key"], "courses.section/en");
    let id = preview.body["id"].as_str().unwrap().to_owned();
    let applied = site
        .call(
            Method::POST,
            &format!("/api/v1/customisation-imports/{id}/apply"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(applied.status, StatusCode::OK, "{}", applied.body);
    let term = site
        .call(
            Method::GET,
            &format!("/api/v1/scopes/{scope}/terms/courses.section"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(term.body["origin"]["scope"], scope.as_str());
    site.drop().await;
}
