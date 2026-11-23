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

/// A placement of a widget on a rectangle of the grid: column, row, width and height.
fn placement(at: [u16; 4], widget: &str, settings: Value) -> Placement {
    let [x, y, w, h] = at;
    Placement {
        widget: widget.to_owned(),
        x,
        y,
        w,
        h,
        pinned: false,
        locked: false,
        held: false,
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
async fn a_layout_keeps_to_the_grid_and_the_sizes_of_its_widgets_and_fills_default_settings() {
    let w = world().await;
    let c = &w.customise;
    let empty = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert_eq!((empty.placements.len(), empty.origin), (0, Origin::Declared));
    assert_eq!((empty.grid.columns, empty.grid.row_pixels), (12, 72));
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
                placement([6, 0, 3, 2], "testing.clock", json!({})),
                placement([0, 0, 3, 2], "testing.clock", json!({ "seconds": true })),
            ],
            false,
        )
        .await
        .unwrap();
    assert_eq!(layout.placements[0].x, 0, "widgets are read in reading order");
    assert_eq!(layout.placements[1].settings["seconds"], json!(false));
    let below = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert_eq!(
        below.origin,
        Origin::Scope {
            scope: w.category,
            locked: false
        }
    );
    let ok = |at| placement(at, "testing.clock", json!({}));
    for wrong in [
        vec![placement([0, 0, 3, 2], "testing.unknown", json!({}))],
        vec![placement([0, 0, 3, 2], "testing.elsewhere", json!({}))],
        vec![placement(
            [0, 0, 3, 2],
            "testing.clock",
            json!({ "seconds": "yes" }),
        )],
        vec![placement(
            [0, 0, 3, 2],
            "testing.clock",
            json!({ "colour": "red" }),
        )],
        vec![ok([0, 0, 7, 2])],
        vec![ok([0, 0, 3, 1])],
        vec![ok([10, 0, 3, 2])],
        vec![ok([0, 0, 3, 2]), ok([2, 1, 3, 2])],
        vec![
            placement([0, 0, 3, 2], "testing.banner", json!({})),
            placement([4, 0, 3, 2], "testing.banner", json!({})),
        ],
    ] {
        let refused = c
            .set_layout(&w.admin, "testing.page", w.course, wrong, false)
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
        vec![placement([0, 0, 3, 2], "testing.clock", json!({}))],
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
        vec![placement([0, 0, 3, 2], "testing.clock", json!({}))],
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

#[tokio::test]
async fn a_widget_a_page_holds_once_shows_once_even_when_an_old_layout_holds_more() {
    let w = world().await;
    let c = &w.customise;
    c.set_layout(
        &w.admin,
        "testing.page",
        w.category,
        vec![placement([0, 0, 3, 2], "testing.banner", json!({}))],
        false,
    )
    .await
    .unwrap();
    // A layout written before the widget was single: a second banner, added behind the service's back.
    let schema = w.db.prefix().schema("customise");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "insert into {schema}.\"placement\" (id, layout_id, widget_code, x, y, w, h, settings)
         select $1, layout_id, widget_code, 4, 0, 3, 2, settings from {schema}.\"placement\" limit 1"
    )))
    .bind(uuid::Uuid::now_v7())
    .execute(w.db.pool())
    .await
    .unwrap();
    let read = c.layout(&w.admin, "testing.page", w.category).await.unwrap();
    let banners = read
        .placements
        .iter()
        .filter(|p| p.widget == "testing.banner")
        .count();
    assert_eq!(banners, 1, "the first in reading order shows");
    assert_eq!(read.placements[0].x, 0);
    w.drop().await;
}

#[tokio::test]
async fn a_widget_every_page_holds_cannot_be_left_out_and_comes_back_when_it_is_missing() {
    let w = world().await;
    let c = &w.customise;
    let refused = c
        .set_layout(
            &w.admin,
            "testing.lesson",
            w.category,
            vec![placement([0, 0, 3, 2], "testing.clock", json!({}))],
            false,
        )
        .await
        .unwrap_err();
    assert_eq!(
        refused.code(),
        "customise.layout_invalid",
        "the anchor cannot be left out"
    );
    let grown = c
        .set_layout(
            &w.admin,
            "testing.lesson",
            w.category,
            vec![placement([0, 0, 12, 4], "testing.anchor", json!({}))],
            false,
        )
        .await
        .unwrap_err();
    assert_eq!(grown.code(), "customise.layout_invalid", "nor take another size");
    let moved = c
        .set_layout(
            &w.admin,
            "testing.lesson",
            w.category,
            vec![placement([4, 2, 8, 4], "testing.anchor", json!({}))],
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        (moved.placements[0].x, moved.placements[0].y),
        (4, 0),
        "but it moves, and rises to the top"
    );
    // A layout written before the anchor was required: it is read with the anchor in the first room that holds it.
    let schema = w.db.prefix().schema("customise");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "update {schema}.\"placement\" set widget_code = 'testing.clock', w = 3, h = 2, x = 0, y = 0"
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    let read = c.layout(&w.admin, "testing.lesson", w.category).await.unwrap();
    let anchor = read
        .placements
        .iter()
        .find(|p| p.widget == "testing.anchor")
        .expect("the anchor is back");
    assert_eq!((anchor.x, anchor.y, anchor.w, anchor.h), (3, 0, 8, 4));
    w.drop().await;
}

#[tokio::test]
async fn widgets_rise_so_no_row_is_left_empty_unless_one_is_pinned() {
    let w = world().await;
    let c = &w.customise;
    let gap = c
        .set_layout(
            &w.admin,
            "testing.page",
            w.category,
            vec![
                placement([0, 0, 3, 2], "testing.clock", json!({})),
                placement([0, 6, 3, 2], "testing.clock", json!({})),
                placement([6, 9, 3, 2], "testing.clock", json!({})),
            ],
            false,
        )
        .await
        .unwrap();
    let rows: Vec<_> = gap.placements.iter().map(|p| (p.x, p.y)).collect();
    assert_eq!(
        rows,
        [(0, 0), (6, 0), (0, 2)],
        "each rises to the widgets above it"
    );
    let mut pinned = placement([0, 6, 3, 2], "testing.clock", json!({}));
    pinned.pinned = true;
    let held = c
        .set_layout(
            &w.admin,
            "testing.page",
            w.category,
            vec![placement([0, 0, 3, 2], "testing.clock", json!({})), pinned],
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        (held.placements[1].y, held.placements[1].pinned),
        (6, true),
        "a pinned widget keeps its row"
    );
    w.drop().await;
}

#[tokio::test]
async fn settings_take_a_text_per_language_may_be_required_and_keep_rich_text_sanitised() {
    let w = world().await;
    let c = &w.customise;
    let note = |settings: Value| placement([0, 0, 3, 2], "testing.note", settings);
    let layout = c
        .set_layout(
            &w.admin,
            "testing.page",
            w.category,
            vec![note(json!({
                "title": { "en": "Welcome", "es": "Bienvenida" },
                "body": { "en": "<p>Hello <strong>all</strong><script>alert(1)</script></p>", "es": "<p onclick=\"x()\">Hola</p>" }
            }))],
            false,
        )
        .await
        .unwrap();
    let settings = &layout.placements[0].settings;
    assert_eq!(
        settings["title"]["es"],
        json!("Bienvenida"),
        "a text per language"
    );
    assert_eq!(
        settings["body"]["en"],
        json!("<p>Hello <strong>all</strong></p>"),
        "nothing that runs is kept"
    );
    assert_eq!(settings["body"]["es"], json!("<p>Hola</p>"));
    for wrong in [
        json!({ "title": "" }),
        json!({}),
        json!({ "title": { "en": "  " } }),
        json!({ "title": { "english": "Hi" } }),
        json!({ "title": "Hi", "body": { "en": 3 } }),
    ] {
        let refused = c
            .set_layout(
                &w.admin,
                "testing.page",
                w.course,
                vec![note(wrong.clone())],
                false,
            )
            .await
            .unwrap_err();
        assert_eq!(refused.code(), "customise.layout_invalid", "{wrong}");
    }
    let flat = c
        .set_layout(
            &w.admin,
            "testing.page",
            w.course,
            vec![placement(
                [0, 0, 3, 2],
                "testing.clock",
                json!({ "seconds": { "en": true } }),
            )],
            false,
        )
        .await
        .unwrap_err();
    assert_eq!(
        flat.code(),
        "customise.layout_invalid",
        "only a translatable setting takes languages"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_widget_locked_by_a_scope_shows_below_as_placed_there_and_is_never_changed_there() {
    let w = world().await;
    let c = &w.customise;
    let mut clock = placement([0, 0, 3, 2], "testing.clock", json!({ "seconds": true }));
    clock.locked = true;
    c.set_layout(&w.admin, "testing.page", w.category, vec![clock.clone()], false)
        .await
        .unwrap();
    // The course arranges its own page, moving what it was given of the clock: that part is the category's.
    let mut moved = Placement {
        held: true,
        x: 8,
        ..clock.clone()
    };
    moved.settings.insert("seconds".to_owned(), json!(false));
    let own = c
        .set_layout(
            &w.admin,
            "testing.page",
            w.course,
            vec![moved, placement([4, 0, 3, 2], "testing.banner", json!({}))],
            false,
        )
        .await
        .unwrap();
    let held = own
        .placements
        .iter()
        .find(|p| p.widget == "testing.clock")
        .expect("the clock is there");
    assert_eq!(
        (held.x, held.y, held.held, held.pinned),
        (0, 0, true, true),
        "as the category placed it, pinned"
    );
    assert_eq!(
        held.settings["seconds"],
        json!(true),
        "with the category's settings"
    );
    assert!(
        own.placements.iter().any(|p| p.widget == "testing.banner"),
        "beside what the course placed"
    );
    assert!(
        !own.lockable,
        "a course has no scopes below that arrange its page"
    );
    let category = c.layout(&w.admin, "testing.page", w.category).await.unwrap();
    assert!(
        category.lockable,
        "a category locks widgets for the courses in it"
    );
    // A section with no layout of its own reads the course's, with the clock still held.
    let below = c.layout(&w.admin, "testing.page", w.section).await.unwrap();
    assert!(
        below
            .placements
            .iter()
            .any(|p| p.widget == "testing.clock" && p.held)
    );
    // On a page each person arranges, the campus locks for every person, and a person locks nothing.
    c.set_layout(&w.admin, "testing.home", w.campus, vec![clock], false)
        .await
        .unwrap();
    c.set_layout(&w.person, "testing.home", w.person_scope, Vec::new(), false)
        .await
        .unwrap();
    let home = c.layout(&w.person, "testing.home", w.person_scope).await.unwrap();
    assert!(
        home.placements
            .iter()
            .any(|p| p.widget == "testing.clock" && p.held),
        "the person keeps it"
    );
    assert!(!home.lockable, "a person's own home has no scopes below");
    w.drop().await;
}
