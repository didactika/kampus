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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of customisation, against PostgreSQL: resolution of terms and flags through the
//! scope tree down to sections with locks, typed custom fields and their values, and permissions.

use kampus_common::ErrorKind;
use kampus_customise::{FieldSpec, FieldType, FieldVisibility, Origin};
use serde_json::{Value, json};

use crate::world::world;

#[tokio::test]
async fn a_term_comes_from_its_module_until_a_scope_renames_it_in_a_language() {
    let w = world().await;
    let c = &w.customise;
    let term = c.term("testing.course", w.section, "es").await.unwrap();
    assert_eq!((term.singular.as_str(), term.origin), ("curso", Origin::Declared));
    let fallback = c.term("testing.course", w.section, "fr").await.unwrap();
    assert_eq!(fallback.singular, "course");
    c.rename_term(
        &w.admin,
        "testing.course",
        w.category,
        "es",
        ("programa", "programas"),
        false,
    )
    .await
    .unwrap();
    let term = c.term("testing.course", w.section, "es").await.unwrap();
    assert_eq!(term.plural, "programas");
    assert_eq!(
        term.origin,
        Origin::Scope {
            scope: w.category,
            locked: false
        }
    );
    assert_eq!(
        c.term("testing.course", w.section, "en").await.unwrap().singular,
        "course"
    );
    c.reset_term(&w.admin, "testing.course", w.category, "es")
        .await
        .unwrap();
    let term = c.term("testing.course", w.section, "es").await.unwrap();
    assert_eq!(term.origin, Origin::Declared);
    w.drop().await;
}

#[tokio::test]
async fn a_lock_above_keeps_its_word_and_refuses_renames_below() {
    let w = world().await;
    let c = &w.customise;
    c.rename_term(
        &w.admin,
        "testing.course",
        w.section,
        "en",
        ("unit", "units"),
        false,
    )
    .await
    .unwrap();
    assert_eq!(
        c.term("testing.course", w.section, "en").await.unwrap().singular,
        "unit"
    );
    c.rename_term(
        &w.admin,
        "testing.course",
        w.campus,
        "en",
        ("module", "modules"),
        true,
    )
    .await
    .unwrap();
    let term = c.term("testing.course", w.section, "en").await.unwrap();
    assert_eq!(term.singular, "module");
    let refused = c
        .rename_term(
            &w.admin,
            "testing.course",
            w.course,
            "en",
            ("subject", "subjects"),
            false,
        )
        .await
        .unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::Conflict);
    let invalid = c
        .rename_term(&w.admin, "testing.course", w.campus, "en", (" ", "x"), false)
        .await
        .unwrap_err();
    assert_eq!(invalid.kind(), ErrorKind::Validation);
    w.drop().await;
}

#[tokio::test]
async fn a_flag_resolves_down_to_sections_and_guards_what_it_offers() {
    let w = world().await;
    let c = &w.customise;
    assert!(c.flag("testing.groups", w.section).await.unwrap().enabled);
    c.require_flag("testing.groups", w.section).await.unwrap();
    c.set_flag(&w.admin, "testing.groups", w.course, false, false)
        .await
        .unwrap();
    let off = c.require_flag("testing.groups", w.section).await.unwrap_err();
    assert_eq!(off.kind(), ErrorKind::Conflict);
    assert!(c.flag("testing.groups", w.category).await.unwrap().enabled);
    c.set_flag(&w.admin, "testing.groups", w.section, true, false)
        .await
        .unwrap();
    assert!(c.flag("testing.groups", w.section).await.unwrap().enabled);
    c.set_flag(&w.admin, "testing.groups", w.category, false, true)
        .await
        .unwrap();
    let flag = c.flag("testing.groups", w.section).await.unwrap();
    assert!(!flag.enabled);
    assert_eq!(
        flag.origin,
        Origin::Scope {
            scope: w.category,
            locked: true
        }
    );
    let flags = c.flags(w.section).await.unwrap();
    assert_eq!(flags.len(), 1);
    let unknown = c.flag("testing.nothing", w.section).await.unwrap_err();
    assert_eq!(unknown.kind(), ErrorKind::NotFound);
    w.drop().await;
}

#[tokio::test]
async fn changing_a_customisation_needs_its_permission() {
    let w = world().await;
    let c = &w.customise;
    let denied = c
        .set_flag(&w.nobody, "testing.groups", w.course, false, false)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let denied = c
        .rename_term(&w.nobody, "testing.course", w.course, "en", ("a", "b"), false)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let field = w.field("level", FieldType::Integer, &[]).await;
    let denied = c.delete_field(&w.nobody, field.id).await.unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn fields_apply_below_their_scope_and_the_nearest_with_a_code_wins() {
    let w = world().await;
    let c = &w.customise;
    let campus_level = w.field("level", FieldType::Integer, &[]).await;
    let mut spec = FieldSpec {
        code: "level".to_owned(),
        name: "Course level".to_owned(),
        field_type: FieldType::Text,
        required: false,
        visibility: FieldVisibility::Private,
        max_length: None,
        choices: Vec::new(),
    };
    let course_level = c
        .create_field(&w.admin, "testing.thing", w.course, spec.clone())
        .await
        .unwrap();
    let taken = c
        .create_field(&w.admin, "testing.thing", w.course, spec.clone())
        .await
        .unwrap_err();
    assert_eq!(taken.kind(), ErrorKind::Conflict);
    spec.code = "Bad Code".to_owned();
    let invalid = c
        .create_field(&w.admin, "testing.thing", w.course, spec)
        .await
        .unwrap_err();
    assert_eq!(invalid.kind(), ErrorKind::Validation);
    let in_section = c.fields_for("testing.thing", w.section).await.unwrap();
    assert_eq!(
        in_section.iter().map(|f| f.id).collect::<Vec<_>>(),
        vec![course_level.id]
    );
    let in_category = c.fields_for("testing.thing", w.category).await.unwrap();
    assert_eq!(
        in_category.iter().map(|f| f.id).collect::<Vec<_>>(),
        vec![campus_level.id]
    );
    assert!(c.sees_values(&w.admin, &course_level, w.course).await.unwrap());
    assert!(!c.sees_values(&w.nobody, &course_level, w.course).await.unwrap());
    c.delete_field(&w.admin, course_level.id).await.unwrap();
    assert_eq!(c.fields_for("testing.thing", w.section).await.unwrap().len(), 1);
    w.drop().await;
}

#[tokio::test]
async fn every_field_type_keeps_only_values_that_fit_it() {
    let w = world().await;
    let text = w.field("motto", FieldType::Text, &[]).await;
    assert_eq!(w.write(&text, &json!("short")).await.unwrap(), json!("short"));
    assert!(w.write(&text, &json!("too long")).await.is_err());
    let integer = w.field("credits", FieldType::Integer, &[]).await;
    assert_eq!(w.write(&integer, &json!(6)).await.unwrap(), json!(6));
    assert!(w.write(&integer, &json!("six")).await.is_err());
    let date = w.field("starts", FieldType::Date, &[]).await;
    assert_eq!(
        w.write(&date, &json!("2026-09-01")).await.unwrap(),
        json!("2026-09-01")
    );
    assert!(w.write(&date, &json!("2026-13-01")).await.is_err());
    let boolean = w.field("online", FieldType::Boolean, &[]).await;
    assert_eq!(w.write(&boolean, &json!(true)).await.unwrap(), json!(true));
    let choice = w.field("mode", FieldType::Choice, &["day", "night"]).await;
    assert_eq!(w.write(&choice, &json!("night")).await.unwrap(), json!("night"));
    assert!(w.write(&choice, &json!("noon")).await.is_err());
    let choices = w.field("tags", FieldType::Choices, &["a", "b", "c"]).await;
    assert_eq!(
        w.write(&choices, &json!(["c", "a"])).await.unwrap(),
        json!(["a", "c"])
    );
    let account = w.field("tutor", FieldType::Account, &[]).await;
    let missing = w.write(&account, &json!(uuid::Uuid::now_v7())).await.unwrap_err();
    assert_eq!(missing.kind(), ErrorKind::Validation);
    assert_eq!(w.write(&text, &Value::Null).await.unwrap(), Value::Null);
    w.drop().await;
}
