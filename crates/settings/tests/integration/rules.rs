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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of settings, against PostgreSQL: resolution through the scope tree, locks,
//! levels, kinds, permissions, and the record and event of every change.

use std::sync::Arc;

use kampus_access::{Access, Actor, PermissionDeclaration, PermissionKind, PrincipalId, ScopeId, ScopeKind};
use kampus_common::{ErrorKind, SystemClock};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_module::{Module, SettingDeclaration, ValueKind};
use kampus_settings::{Origin, Settings};
use serde_json::json;

/// The permission that sets the settings of these tests.
const MANAGE: &str = "testing.settings.manage";

/// The permissions of these tests.
const PERMISSIONS: &[PermissionDeclaration] = &[PermissionDeclaration {
    code: MANAGE,
    kind: PermissionKind::Write,
    risks: &[],
    lowest_kind: ScopeKind::Course,
}];

/// The settings of these tests.
const SETTINGS: &[SettingDeclaration] = &[
    SettingDeclaration {
        key: "testing.limit",
        summary: "A limit",
        group: "testing",
        kind: ValueKind::Integer {
            min: 1,
            max: 100,
            default: 10,
        },
        levels: &[
            ScopeKind::Platform,
            ScopeKind::Campus,
            ScopeKind::Category,
            ScopeKind::Course,
        ],
        permission: MANAGE,
        restart_required: false,
    },
    SettingDeclaration {
        key: "testing.platform_only",
        summary: "Only for the platform",
        group: "testing",
        kind: ValueKind::Boolean { default: false },
        levels: &[ScopeKind::Platform],
        permission: MANAGE,
        restart_required: true,
    },
];

/// A module that declares the settings of these tests.
const TESTING: Module = Module {
    name: "testing",
    permissions: PERMISSIONS,
    settings: SETTINGS,
    ..kampus_settings::MODULE
};

/// A tree of scopes with its administrator.
struct World {
    /// The service.
    settings: Settings,
    /// The database.
    db: Database,
    /// The platform.
    platform: ScopeId,
    /// A campus.
    campus: ScopeId,
    /// A category of the campus.
    category: ScopeId,
    /// A course of the category.
    course: ScopeId,
    /// The campus administrator.
    admin: Actor,
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("s{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let modules = [
        &kampus_events::MODULE,
        &kampus_access::MODULE,
        &kampus_settings::MODULE,
    ];
    let schemas: Vec<_> = modules.iter().map(|m| m.schema()).collect();
    Upgrader::new(&db, &schemas, "0.1.0").apply(true).await.unwrap();
    let access = Access::new(db.clone(), Arc::new(SystemClock));
    access
        .sync_modules(&[&kampus_access::MODULE, &TESTING])
        .await
        .unwrap();
    kampus_events::Events::new(db.clone())
        .sync_declarations(&[&kampus_settings::MODULE])
        .await
        .unwrap();
    let settings = Settings::new(db.clone(), access.clone());
    settings.sync_declarations(&[&TESTING]).await.unwrap();
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let category = access.create_scope(campus, ScopeKind::Category).await.unwrap();
    let course = access.create_scope(category, ScopeKind::Course).await.unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();
    let admin = PrincipalId::generate();
    access.create_principal(admin, campus).await.unwrap();
    access
        .grant_for_module("setup", roles["campus_administrator"], campus, admin, None)
        .await
        .unwrap();
    access
        .set_role_value(
            &Actor::Principal(admin),
            roles["campus_administrator"],
            MANAGE,
            Some(kampus_access::Value::Allow),
        )
        .await
        .unwrap();
    World {
        settings,
        db,
        platform,
        campus,
        category,
        course,
        admin: Actor::Principal(admin),
    }
}

impl World {
    /// Drops the installation.
    async fn drop(self) {
        for module in ["settings", "access", "events", "db"] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
        self.db.close().await;
    }
}

#[tokio::test]
async fn the_nearest_scope_that_sets_a_value_wins_and_says_where_it_came_from() {
    let w = world().await;
    let s = &w.settings;
    let read = s.resolve("testing.limit", w.course).await.unwrap();
    assert_eq!((read.value, read.origin), (json!(10), Origin::Default));
    s.set_as_operator("testing.limit", w.platform, Some(json!(20)), false)
        .await
        .unwrap();
    s.set(&w.admin, "testing.limit", w.category, json!(30), false, None)
        .await
        .unwrap();
    let read = s.resolve("testing.limit", w.course).await.unwrap();
    assert_eq!(read.value, json!(30));
    assert_eq!(
        read.origin,
        Origin::Scope {
            scope: w.category,
            locked: false
        }
    );
    assert_eq!(
        s.resolve("testing.limit", w.campus).await.unwrap().value,
        json!(20)
    );
    s.unset(&w.admin, "testing.limit", w.category).await.unwrap();
    assert_eq!(
        s.resolve("testing.limit", w.course).await.unwrap().value,
        json!(20)
    );
    let all = s.resolve_prefix("testing.", w.course).await.unwrap();
    assert_eq!(all.len(), 2);
    w.drop().await;
}

#[tokio::test]
async fn a_locked_value_holds_for_every_scope_below() {
    let w = world().await;
    let s = &w.settings;
    s.set(&w.admin, "testing.limit", w.course, json!(50), false, None)
        .await
        .unwrap();
    s.set(&w.admin, "testing.limit", w.campus, json!(5), true, None)
        .await
        .unwrap();
    let read = s.resolve("testing.limit", w.course).await.unwrap();
    assert_eq!(
        read.value,
        json!(5),
        "a lock above wins over a value set below before it"
    );
    assert_eq!(
        read.origin,
        Origin::Scope {
            scope: w.campus,
            locked: true
        }
    );
    let refused = s
        .set(&w.admin, "testing.limit", w.category, json!(7), false, None)
        .await
        .unwrap_err();
    assert_eq!(refused.code(), "settings.locked_above");
    w.drop().await;
}

#[tokio::test]
async fn values_keep_to_their_kind_their_levels_and_their_permission() {
    let w = world().await;
    let s = &w.settings;
    assert_eq!(
        s.set(&w.admin, "testing.limit", w.course, json!(1000), false, None)
            .await
            .unwrap_err()
            .code(),
        "settings.value_invalid"
    );
    assert_eq!(
        s.set(&w.admin, "testing.limit", w.course, json!("ten"), false, None)
            .await
            .unwrap_err()
            .code(),
        "settings.value_invalid"
    );
    assert_eq!(
        s.set(
            &w.admin,
            "testing.platform_only",
            w.campus,
            json!(true),
            false,
            None
        )
        .await
        .unwrap_err()
        .code(),
        "settings.level_invalid"
    );
    assert_eq!(
        s.set(&w.admin, "testing.nothing", w.campus, json!(1), false, None)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    let stranger = Actor::Principal(PrincipalId::generate());
    assert_eq!(
        s.set(&stranger, "testing.limit", w.course, json!(3), false, None)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}

#[tokio::test]
async fn every_change_is_recorded_and_published() {
    let w = world().await;
    let s = &w.settings;
    s.set(&w.admin, "testing.limit", w.course, json!(40), false, None)
        .await
        .unwrap();
    s.set(&w.admin, "testing.limit", w.course, json!(45), false, None)
        .await
        .unwrap();
    s.unset(&w.admin, "testing.limit", w.course).await.unwrap();
    let history = s.history("testing.limit", w.course).await.unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].before, Some(json!(45)));
    assert_eq!(history[0].after, None);
    assert_eq!(history[2].before, None);
    let published: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {}.\"event\" where type_code = 'settings.changed'",
        w.db.prefix().schema("events")
    )))
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(published, 3);
    w.drop().await;
}
