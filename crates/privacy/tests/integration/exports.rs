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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Exports of personal data: a person asks for their own, someone who handles privacy asks for another
//! person's, every module answers for its own data, and the package is for the person who asked, for a while.

use kampus_common::ErrorKind;
use kampus_module::ServiceContext;
use kampus_progress::RuleKind;
use kampus_records::{GradeWrite, RecordFields, Source};
use rust_decimal::Decimal;

use crate::world::{World, who, world};

/// The services the handlers read.
fn services(w: &World) -> ServiceContext {
    ServiceContext::default()
        .with(w.records.clone())
        .with(w.progress.clone())
        .with(w.privacy.clone())
}

/// Gives lea a grade and a state, so there is something to export.
async fn something_to_export(w: &World) {
    let page = w.page("Report").await;
    let record = w
        .records
        .define_record(
            &w.admin,
            w.course.id,
            None,
            RecordFields::new("Quiz", Decimal::TEN),
        )
        .await
        .unwrap();
    w.records
        .set_grades(
            &w.admin,
            record.id,
            vec![GradeWrite {
                account: who(&w.lea),
                raw: Some(Decimal::from(7)),
                text: None,
                feedback: Some("good".to_owned()),
            }],
            Source::Teacher,
        )
        .await
        .unwrap();
    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::View])
        .await
        .unwrap();
    w.progress.view(&w.lea, page.id).await.unwrap();
}

#[tokio::test]
async fn a_person_exports_their_data_and_every_module_answers_for_its_own() {
    let w = world().await;
    something_to_export(&w).await;
    let request = w.privacy.request_export(&w.lea, who(&w.lea)).await.unwrap();
    assert_eq!(request.state, "pending");
    let categories: Vec<_> = request.items.iter().map(|i| i.category.as_str()).collect();
    assert_eq!(categories, ["progress.activity", "records.grades"]);
    assert_eq!(
        w.privacy.package(&w.lea, request.id).await.unwrap_err().code(),
        "privacy.package_not_ready"
    );

    let answered = w.privacy.run(&services(&w)).await.unwrap();
    assert_eq!(answered, 2);
    let done = w.privacy.request(&w.lea, request.id).await.unwrap();
    assert_eq!(done.state, "completed");
    assert!(done.expires_at.is_some());

    let package = w.privacy.package(&w.lea, request.id).await.unwrap();
    assert_eq!(
        package["data"]["records.grades"]["grade"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(package["data"]["records.grades"]["entry"][0]["feedback"], "good");
    assert_eq!(
        package["data"]["progress.activity"]["activity_state"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(w.privacy.requests(&w.lea).await.unwrap().len(), 1);
    w.drop().await;
}

#[tokio::test]
async fn others_need_the_permission_and_never_read_the_package_of_someone_else() {
    let w = world().await;
    something_to_export(&w).await;
    let denied = w.privacy.request_export(&w.max, who(&w.lea)).await.unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);

    let by_admin = w.privacy.request_export(&w.admin, who(&w.lea)).await.unwrap();
    assert_eq!(
        w.privacy.request(&w.admin, by_admin.id).await.unwrap().account,
        who(&w.lea)
    );
    assert_eq!(
        w.privacy.request(&w.max, by_admin.id).await.unwrap_err().kind(),
        ErrorKind::NotFound,
        "nobody else follows the request"
    );
    w.privacy.run(&services(&w)).await.unwrap();
    assert!(w.privacy.package(&w.admin, by_admin.id).await.is_ok());
    assert_eq!(
        w.privacy.package(&w.lea, by_admin.id).await.unwrap_err().kind(),
        ErrorKind::NotFound,
        "the package is for the person who asked"
    );
    w.drop().await;
}

#[tokio::test]
async fn an_exported_package_is_forgotten_when_its_time_ends() {
    let w = world().await;
    something_to_export(&w).await;
    let request = w.privacy.request_export(&w.lea, who(&w.lea)).await.unwrap();
    w.privacy.run(&services(&w)).await.unwrap();
    assert_eq!(w.privacy.expire().await.unwrap(), 0, "not yet");

    sqlx::query(sqlx::AssertSqlSafe(format!(
        "update {}.\"request\" set expires_at = now() - interval '1 minute'",
        w.db.prefix().schema("privacy")
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    assert_eq!(w.privacy.expire().await.unwrap(), 1);
    assert_eq!(
        w.privacy.package(&w.lea, request.id).await.unwrap_err().code(),
        "privacy.package_expired"
    );
    w.drop().await;
}
