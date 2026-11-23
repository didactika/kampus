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

//! One test per behaviour of packages, against PostgreSQL: exporting what a scope sets, previewing the
//! differences of an import, applying it in one transaction once, and refusing what does not fit.

use std::collections::BTreeMap;

use kampus_common::ErrorKind;
use kampus_customise::{
    Choice, FieldSpec, FieldType, FieldVisibility, MenuItem, Package, PackageKind, Placement, RuleAction,
    RuleSpec,
};
use serde_json::json;

use crate::world::{World, world};

/// Every kind.
const ALL: &[PackageKind] = &[
    PackageKind::Terms,
    PackageKind::Flags,
    PackageKind::Fields,
    PackageKind::Layouts,
    PackageKind::Menus,
    PackageKind::Theme,
    PackageKind::Rules,
];

/// Sets one of everything in the category.
async fn customised(w: &World) {
    let c = &w.customise;
    c.rename_term(
        &w.admin,
        "testing.course",
        w.category,
        "es",
        ("asignatura", "asignaturas"),
        false,
    )
    .await
    .unwrap();
    c.set_flag(&w.admin, "testing.groups", w.category, false, false)
        .await
        .unwrap();
    c.create_field(
        &w.admin,
        "testing.thing",
        w.category,
        FieldSpec {
            code: "mode".to_owned(),
            name: "Mode".to_owned(),
            field_type: FieldType::Choice,
            required: false,
            visibility: FieldVisibility::Public,
            max_length: None,
            choices: vec![Choice {
                code: "day".to_owned(),
                label: "Day".to_owned(),
            }],
        },
    )
    .await
    .unwrap();
    let clock = Placement {
        widget: "testing.clock".to_owned(),
        x: 0,
        y: 0,
        w: 3,
        h: 2,
        pinned: false,
        locked: false,
        held: false,
        grows: false,
        settings: BTreeMap::new(),
        permission: None,
    };
    c.set_layout(&w.admin, "testing.page", w.category, vec![clock], false)
        .await
        .unwrap();
    let item = MenuItem {
        label: "Home".to_owned(),
        target: "/".to_owned(),
        permission: None,
        children: Vec::new(),
    };
    c.set_menu(&w.admin, "testing.menu", w.category, vec![item], false)
        .await
        .unwrap();
    let tokens = serde_json::from_value(json!({ "primary": "#0f766e", "radius": 12 })).unwrap();
    c.set_theme(&w.admin, w.category, &tokens, false).await.unwrap();
    let rule = RuleSpec {
        name: "Note".to_owned(),
        trigger: "testing.happened".to_owned(),
        conditions: None,
        actions: vec![RuleAction {
            action: "testing.note".to_owned(),
            params: serde_json::from_value(json!({ "tag": "imported" })).unwrap(),
        }],
        enabled: true,
    };
    c.create_rule(&w.admin, w.category, rule).await.unwrap();
}

#[tokio::test]
async fn a_package_exports_what_a_scope_sets_and_imports_once_into_another() {
    let w = world().await;
    let c = &w.customise;
    customised(&w).await;
    let exported = c.export(&w.admin, w.category, ALL).await.unwrap();
    let package = &exported.package;
    assert_eq!(
        (
            package.terms.len(),
            package.flags.len(),
            package.fields.len(),
            package.layouts.len()
        ),
        (1, 1, 1, 1)
    );
    assert_eq!((package.menus.len(), package.rules.len()), (1, 1));
    assert_eq!(package.theme.as_ref().unwrap().tokens["radius"], json!(12));
    assert_eq!(exported.checksum.len(), 64);
    let only_terms = c
        .export(&w.admin, w.category, &[PackageKind::Terms])
        .await
        .unwrap();
    assert_eq!(only_terms.package.flags.len(), 0);

    let import = c
        .preview_import(&w.admin, w.course, package.clone())
        .await
        .unwrap();
    assert_eq!(import.differences.len(), 7);
    assert!(import.differences.iter().all(|d| d.before.is_none()));
    assert_eq!(import.checksum, exported.checksum);
    let applied = c.apply_import(&w.admin, import.id).await.unwrap();
    assert!(applied.applied_at.is_some());
    let again = c.apply_import(&w.admin, import.id).await.unwrap_err();
    assert_eq!(again.code(), "customise.import_applied");
    let copied = c.export(&w.admin, w.course, ALL).await.unwrap();
    assert_eq!(copied.checksum, exported.checksum, "the course now sets the same");
    let same = c
        .preview_import(&w.admin, w.course, package.clone())
        .await
        .unwrap();
    assert_eq!(same.differences, []);

    let mut changed = package.clone();
    changed.terms[0].plural = "materias".to_owned();
    let diff = c.preview_import(&w.admin, w.course, changed).await.unwrap();
    assert_eq!(diff.differences.len(), 1);
    assert_eq!(diff.differences[0].key, "testing.course/es");
    assert!(diff.differences[0].before.is_some());
    w.drop().await;
}

#[tokio::test]
async fn an_import_that_does_not_fit_the_scope_is_refused_whole() {
    let w = world().await;
    let c = &w.customise;
    let mut package = Package::empty();
    package.terms = serde_json::from_value(json!([
        { "code": "testing.unknown", "language": "en", "singular": "a", "plural": "b" }
    ]))
    .unwrap();
    package.layouts = serde_json::from_value(json!([
        { "page_kind": "testing.page", "placements": [{ "widget": "testing.unknown", "x": 0, "y": 0, "w": 3, "h": 2 }] }
    ]))
    .unwrap();
    package.theme = serde_json::from_value(json!({ "tokens": { "primary": "yellow" } })).unwrap();
    package.rules = vec![RuleSpec {
        name: "Bad".to_owned(),
        trigger: "testing.unknown".to_owned(),
        conditions: None,
        actions: vec![],
        enabled: true,
    }];
    let refused = c.preview_import(&w.admin, w.course, package).await.unwrap_err();
    assert_eq!(refused.code(), "customise.package_invalid");
    let fields: Vec<&str> = refused.fields().iter().map(|f| f.field.as_str()).collect();
    for expected in [
        "terms[0].code",
        "layouts[0].placements[0].widget",
        "theme.tokens.primary",
        "rules[0].trigger",
    ] {
        assert!(fields.contains(&expected), "{expected} in {fields:?}");
    }
    let tokens = serde_json::from_value(json!({ "radius": 4 })).unwrap();
    c.set_theme(&w.admin, w.campus, &tokens, true).await.unwrap();
    let mut themed = Package::empty();
    themed.theme = serde_json::from_value(json!({ "tokens": { "radius": 10 } })).unwrap();
    let locked = c
        .preview_import(&w.admin, w.course, themed.clone())
        .await
        .unwrap_err();
    assert_eq!(locked.fields()[0].rule, "locked_above");
    let denied = c.preview_import(&w.nobody, w.course, themed).await.unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}
