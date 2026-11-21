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

//! Retention: a campus says how long each kind of data an erasure keeps stays after the account is deleted,
//! and a job purges it when the time ends, unless a legal hold covers the account.

use kampus_common::ErrorKind;
use kampus_module::ServiceContext;
use kampus_records::{GradeWrite, RecordFields, Source};
use rust_decimal::Decimal;

use crate::world::{World, who, world};

/// The services the handlers and the tombstone need.
fn services(w: &World) -> ServiceContext {
    ServiceContext::default()
        .with(w.records.clone())
        .with(w.progress.clone())
        .with(w.privacy.clone())
        .with(w.identity.clone())
}

/// Gives a grade to lea, deletes her account and returns how many grades she has left.
async fn grade_and_delete_lea(w: &World) {
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
                feedback: None,
            }],
            Source::Teacher,
        )
        .await
        .unwrap();
    w.privacy
        .request_deletion(&w.admin, who(&w.lea), "requested by the person in writing")
        .await
        .unwrap();
    w.privacy.run(&services(w)).await.unwrap();
}

/// How many grades lea has.
async fn grades_of_lea(w: &World) -> usize {
    w.records.personal_data(who(&w.lea)).await.unwrap()["grade"]
        .as_array()
        .unwrap()
        .len()
}

/// Makes the account of lea look deleted a long time ago.
async fn age_the_deletion(w: &World) {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "update {}.\"account\" set deleted_at = now() - interval '400 days' where id = $1",
        w.db.prefix().schema("identity")
    )))
    .bind(who(&w.lea).as_uuid())
    .execute(w.db.pool())
    .await
    .unwrap();
}

#[tokio::test]
async fn the_categories_and_their_retention_are_set_by_who_may() {
    let w = world().await;
    let all = w.privacy.retention(&w.admin).await.unwrap();
    assert!(
        all.iter()
            .any(|r| r.category == "records.grades" && r.keep_days.is_none())
    );
    assert_eq!(
        w.privacy.retention(&w.lea).await.unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
    let set = w
        .privacy
        .set_retention(&w.admin, "records.grades", Some(365))
        .await
        .unwrap();
    assert!(
        set.iter()
            .any(|r| r.category == "records.grades" && r.keep_days == Some(365))
    );
    assert_eq!(
        w.privacy
            .set_retention(&w.admin, "nobody.knows", Some(1))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        w.privacy
            .set_retention(&w.admin, "records.grades", Some(0))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Validation
    );
    let cleared = w
        .privacy
        .set_retention(&w.admin, "records.grades", None)
        .await
        .unwrap();
    assert!(cleared.iter().all(|r| r.keep_days.is_none()));
    w.drop().await;
}

#[tokio::test]
async fn what_an_erasure_keeps_is_purged_when_its_retention_ends() {
    let w = world().await;
    grade_and_delete_lea(&w).await;
    assert_eq!(grades_of_lea(&w).await, 1, "kept for the academic record");

    assert_eq!(w.privacy.retain(&services(&w)).await.unwrap(), 0, "no rule yet");
    w.privacy
        .set_retention(&w.admin, "records.grades", Some(365))
        .await
        .unwrap();
    assert_eq!(
        w.privacy.retain(&services(&w)).await.unwrap(),
        0,
        "the period has not ended"
    );
    age_the_deletion(&w).await;

    assert_eq!(w.privacy.retain(&services(&w)).await.unwrap(), 1);
    assert_eq!(grades_of_lea(&w).await, 0);
    assert_eq!(
        w.privacy.retain(&services(&w)).await.unwrap(),
        0,
        "a purge never repeats"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_legal_hold_postpones_the_purge() {
    let w = world().await;
    grade_and_delete_lea(&w).await;
    w.privacy
        .set_retention(&w.admin, "records.grades", Some(365))
        .await
        .unwrap();
    age_the_deletion(&w).await;
    let hold = w
        .privacy
        .place_hold(&w.admin, None, "audit of the campus", None)
        .await
        .unwrap();
    assert_eq!(w.privacy.retain(&services(&w)).await.unwrap(), 0);
    assert_eq!(grades_of_lea(&w).await, 1);

    w.privacy.release_hold(&w.admin, hold.id).await.unwrap();
    assert_eq!(w.privacy.retain(&services(&w)).await.unwrap(), 1);
    assert_eq!(grades_of_lea(&w).await, 0);
    w.drop().await;
}
