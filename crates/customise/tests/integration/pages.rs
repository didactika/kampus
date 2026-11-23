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

//! One test per behaviour of layouts, menus and themes, against PostgreSQL: validation against what modules
//! declare, resolution through the scope tree with locks, what each reader sees, and theme contrast.

use std::collections::BTreeMap;

use kampus_common::ErrorKind;
use kampus_customise::{MenuItem, Origin, Placement};
use kampus_module::Module;
use serde_json::{Value, json};

use crate::world::{TESTING, world};

/// A placement of a widget in a region.
fn placement(region: &str, widget: &str, settings: Value) -> Placement {
    Placement {
        region: region.to_owned(),
        widget: widget.to_owned(),
        settings: serde_json::from_value(settings).unwrap(),
        permission: None,
    }
}

/// A menu item without children.
fn item(label: &str, target: &str, permission: Option<&str>) -> MenuItem {
    MenuItem {
        label: label.to_owned(),
        target: target.to_owned(),
        permission: permission.map(str::to_owned),
        children: Vec::new(),
    }
}

/// Tokens from a JSON object.
fn tokens(value: Value) -> BTreeMap<String, Value> {
    serde_json::from_value(value).unwrap()
}

#[tokio::test]
async fn a_layout_keeps_to_the_regions_and_widgets_of_its_page_and_fills_default_settings() {
    let w = world().await;
    let c = &w.customise;
    let empty = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert_eq!((empty.placements.len(), empty.origin), (0, Origin::Declared));
    assert_eq!(empty.regions, ["main", "side"]);
    assert!(empty.editable, "an administrator may arrange the page");
    assert!(
        empty.widgets.iter().any(|widget| widget.code == "testing.clock"),
        "and is offered the widgets of the page with their settings"
    );
    let reader = c.layout(&w.nobody, "testing.page", w.section).await.unwrap();
    assert!(
        !reader.editable && reader.widgets.is_empty(),
        "a reader only reads"
    );
    let layout = c
        .set_layout(
            &w.admin,
            "testing.page",
            w.category,
            vec![
                placement("side", "testing.clock", json!({})),
                placement("main", "testing.clock", json!({ "seconds": true })),
            ],
            false,
        )
        .await
        .unwrap();
    assert_eq!(layout.placements[0].region, "main", "regions keep their order");
    assert_eq!(layout.placements[1].settings["seconds"], json!(false));
    let below = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert_eq!(
        below.origin,
        Origin::Scope {
            scope: w.category,
            locked: false
        }
    );
    for wrong in [
        placement("footer", "testing.clock", json!({})),
        placement("main", "testing.unknown", json!({})),
        placement("main", "testing.elsewhere", json!({})),
        placement("main", "testing.clock", json!({ "seconds": "yes" })),
        placement("main", "testing.clock", json!({ "colour": "red" })),
    ] {
        let refused = c
            .set_layout(&w.admin, "testing.page", w.course, vec![wrong], false)
            .await
            .unwrap_err();
        assert_eq!(refused.code(), "customise.layout_invalid");
    }
    let unknown = c.layout(&w.admin, "testing.nothing", w.course).await.unwrap_err();
    assert_eq!(unknown.kind(), ErrorKind::NotFound);
    w.drop().await;
}

#[tokio::test]
async fn each_person_arranges_their_own_home_and_nothing_else() {
    let w = world().await;
    let c = &w.customise;
    c.set_layout(
        &w.admin,
        "testing.home",
        w.campus,
        vec![placement("main", "testing.clock", json!({}))],
        false,
    )
    .await
    .unwrap();
    let inherited = c.layout(&w.person, "testing.home", w.person_scope).await.unwrap();
    assert_eq!(
        inherited.placements.len(),
        1,
        "until the person arranges it, their home is the campus's"
    );
    assert!(inherited.editable, "a person may arrange their own home");
    c.set_layout(&w.person, "testing.home", w.person_scope, Vec::new(), false)
        .await
        .unwrap();
    assert_eq!(
        c.layout(&w.person, "testing.home", w.person_scope)
            .await
            .unwrap()
            .placements
            .len(),
        0,
        "the person emptied their own home"
    );
    assert_eq!(
        c.layout(&w.admin, "testing.home", w.campus)
            .await
            .unwrap()
            .placements
            .len(),
        1,
        "what a person arranges is theirs alone"
    );
    for (page, scope, why) in [
        ("testing.home", w.campus, "not the home of the campus"),
        ("testing.home", w.other_scope, "not the home of another person"),
        ("testing.page", w.person_scope, "not a page that is not personal"),
    ] {
        let denied = c
            .set_layout(&w.person, page, scope, Vec::new(), false)
            .await
            .unwrap_err();
        assert_eq!(denied.kind(), ErrorKind::PermissionDenied, "{why}");
    }
    assert!(
        !c.layout(&w.person, "testing.home", w.other_scope)
            .await
            .unwrap()
            .editable
    );
    c.reset_layout(&w.person, "testing.home", w.person_scope)
        .await
        .unwrap();
    assert_eq!(
        c.layout(&w.person, "testing.home", w.person_scope)
            .await
            .unwrap()
            .placements
            .len(),
        1,
        "back to the campus's"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_locked_layout_holds_below_and_a_widget_no_longer_declared_is_skipped() {
    let w = world().await;
    let c = &w.customise;
    c.set_layout(
        &w.admin,
        "testing.page",
        w.campus,
        vec![placement("main", "testing.clock", json!({}))],
        true,
    )
    .await
    .unwrap();
    let refused = c
        .set_layout(&w.admin, "testing.page", w.course, Vec::new(), false)
        .await
        .unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::Conflict);
    assert!(
        !c.layout(&w.admin, "testing.page", w.course)
            .await
            .unwrap()
            .editable,
        "a lock above leaves nothing to arrange below"
    );
    let denied = c
        .set_layout(&w.nobody, "testing.page", w.campus, Vec::new(), false)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let without_clock = Module {
        widgets: &[],
        ..TESTING
    };
    c.sync_declarations(&[&without_clock]).await.unwrap();
    let layout = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert!(layout.placements.is_empty(), "the page still opens");
    c.reset_layout(&w.admin, "testing.page", w.campus).await.unwrap();
    let reset = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert_eq!(reset.origin, Origin::Declared);
    w.drop().await;
}

#[tokio::test]
async fn a_menu_shows_each_reader_only_the_items_they_may_see() {
    let w = world().await;
    let c = &w.customise;
    let mut admin_only = item("Settings", "/admin", Some("customise.theme.manage"));
    admin_only.children = vec![item("Theme", "/admin/theme", None)];
    let mut help = item("Help", "https://example.org/help", None);
    help.children = vec![item("Contact", "/contact", None)];
    let menu = c
        .set_menu(&w.admin, "testing.menu", w.campus, vec![help, admin_only], false)
        .await
        .unwrap();
    assert_eq!(menu.items.len(), 2);
    assert_eq!(menu.items[1].children[0].label, "Theme");
    let seen = c.menu(&w.nobody, "testing.menu", w.course).await.unwrap();
    assert_eq!(seen.items.len(), 1, "the hidden item takes its children with it");
    assert!(!seen.editable);
    assert!(menu.editable);
    assert_eq!(seen.items[0].children[0].target, "/contact");
    let mut deep = item("1", "/1", None);
    let mut two = item("2", "/2", None);
    let mut three = item("3", "/3", None);
    three.children = vec![item("4", "/4", None)];
    two.children = vec![three];
    deep.children = vec![two];
    for wrong in [
        vec![item(" ", "/empty", None)],
        vec![item("Bad", "javascript:alert(1)", None)],
        vec![item("Bad", "//elsewhere.example", None)],
        vec![deep],
    ] {
        let refused = c
            .set_menu(&w.admin, "testing.menu", w.course, wrong, false)
            .await
            .unwrap_err();
        assert_eq!(refused.code(), "customise.menu_invalid");
    }
    c.reset_menu(&w.admin, "testing.menu", w.campus).await.unwrap();
    let menu = c.menu(&w.admin, "testing.menu", w.course).await.unwrap();
    assert_eq!(menu.items, Vec::<MenuItem>::new());
    w.drop().await;
}

#[tokio::test]
async fn a_theme_merges_tokens_down_the_tree_and_keeps_its_contrast() {
    let w = world().await;
    let c = &w.customise;
    let initial = c.theme(w.course).await.unwrap();
    assert_eq!(initial.tokens["primary"], json!("#0f766e"));
    assert_eq!(initial.origins["primary"], Origin::Declared);
    assert_eq!(initial.light.0["primary"], "#0f766e");
    assert_eq!(initial.dark.0["primary"], "#5eead4");
    c.set_theme(
        &w.admin,
        w.campus,
        &tokens(json!({ "primary": "#1D4ED8" })),
        false,
    )
    .await
    .unwrap();
    c.set_theme(&w.admin, w.category, &tokens(json!({ "radius": 20 })), false)
        .await
        .unwrap();
    let theme = c.theme(w.course).await.unwrap();
    assert_eq!(theme.tokens["primary"], json!("#1d4ed8"));
    assert_eq!(theme.light.0["primary"], "#1d4ed8");
    assert_eq!(theme.tokens["radius"], json!(20));
    assert_eq!(
        theme.origins["radius"],
        Origin::Scope {
            scope: w.category,
            locked: false
        }
    );
    assert!(theme.css().contains("--k-radius: 20px;"));
    assert!(theme.css().contains("--k-color-primary: #1d4ed8;"));
    let yellow = c
        .set_theme(
            &w.admin,
            w.course,
            &tokens(json!({ "primary": "#facc15" })),
            false,
        )
        .await
        .unwrap();
    assert_ne!(
        yellow.light.0["primary"], "#facc15",
        "a seed too light for text gives a darker tone of it"
    );
    assert!(yellow.light.shortfalls().is_empty() && yellow.dark.shortfalls().is_empty());
    for wrong in [
        json!({ "primary": "blue" }),
        json!({ "radius": 99 }),
        json!({ "shadow": "big" }),
        json!({ "density": "huge" }),
        json!({ "font_family": "Arial; } body { display: none" }),
    ] {
        let refused = c
            .set_theme(&w.admin, w.course, &tokens(wrong), false)
            .await
            .unwrap_err();
        assert_eq!(refused.code(), "customise.theme_invalid");
    }
    c.reset_theme(&w.admin, w.course).await.unwrap();
    c.set_theme(&w.admin, w.campus, &tokens(json!({ "primary": "#0f766e" })), true)
        .await
        .unwrap();
    let locked = c
        .set_theme(
            &w.admin,
            w.course,
            &tokens(json!({ "density": "compact" })),
            false,
        )
        .await
        .unwrap_err();
    assert_eq!(locked.kind(), ErrorKind::Conflict);
    assert_eq!(
        c.theme(w.course).await.unwrap().tokens["radius"],
        json!(12),
        "a lock above hides what is below"
    );
    c.reset_theme(&w.admin, w.campus).await.unwrap();
    assert_eq!(c.theme(w.course).await.unwrap().tokens["radius"], json!(20));
    w.drop().await;
}
