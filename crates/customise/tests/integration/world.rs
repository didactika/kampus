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

//! The world of the integration tests: an installation of its own with a tree of scopes down to a section,
//! a module that declares what the tests customise, its administrator and someone without permissions.

use std::sync::Arc;

use kampus_access::{Access, Actor, PrincipalId, ScopeId, ScopeKind};
use kampus_common::SystemClock;
use kampus_customise::{Choice, Customise, FieldSpec, FieldType, FieldVisibility};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_module::{
    FieldKindDeclaration, FlagDeclaration, MenuKindDeclaration, Module, PageKindDeclaration, TermDeclaration,
    TermForms, ValueKind, WidgetDeclaration, WidgetSetting,
};
use serde_json::Value;

/// The terms of these tests.
const TERMS: &[TermDeclaration] = &[TermDeclaration {
    code: "testing.course",
    forms: &[
        TermForms {
            language: "en",
            singular: "course",
            plural: "courses",
        },
        TermForms {
            language: "es",
            singular: "curso",
            plural: "cursos",
        },
    ],
}];

/// The flags of these tests.
const FLAGS: &[FlagDeclaration] = &[FlagDeclaration {
    code: "testing.groups",
    summary: "Groups",
    default: true,
}];

/// The kinds of entity of these tests.
const KINDS: &[FieldKindDeclaration] = &[FieldKindDeclaration {
    code: "testing.thing",
    summary: "A thing",
}];

/// The page kinds of these tests.
const PAGES: &[PageKindDeclaration] = &[PageKindDeclaration {
    code: "testing.page",
    summary: "A page",
    regions: &["main", "side"],
}];

/// The widgets of these tests.
const WIDGETS: &[WidgetDeclaration] = &[
    WidgetDeclaration {
        code: "testing.clock",
        summary: "A clock",
        page_kinds: &["testing.page"],
        settings: &[WidgetSetting {
            key: "seconds",
            kind: ValueKind::Boolean { default: false },
        }],
    },
    WidgetDeclaration {
        code: "testing.elsewhere",
        summary: "A widget for another page",
        page_kinds: &["testing.other"],
        settings: &[],
    },
];

/// The kinds of menu of these tests.
const MENUS: &[MenuKindDeclaration] = &[MenuKindDeclaration {
    code: "testing.menu",
    summary: "A menu",
}];

/// A module that declares what these tests customise.
pub(crate) const TESTING: Module = Module {
    name: "testing",
    terms: TERMS,
    flags: FLAGS,
    field_kinds: KINDS,
    page_kinds: PAGES,
    widgets: WIDGETS,
    menu_kinds: MENUS,
    ..kampus_customise::MODULE
};

/// A tree of scopes down to a section, with its administrator and someone without permissions.
pub(crate) struct World {
    /// The service.
    pub(crate) customise: Customise,
    /// The database.
    pub(crate) db: Database,
    /// A campus.
    pub(crate) campus: ScopeId,
    /// A category of the campus.
    pub(crate) category: ScopeId,
    /// A course of the category.
    pub(crate) course: ScopeId,
    /// A section of the course.
    pub(crate) section: ScopeId,
    /// The campus administrator.
    pub(crate) admin: Actor,
    /// A principal without roles.
    pub(crate) nobody: Actor,
}

/// Builds the world in an installation of its own prefix.
pub(crate) async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("c{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let modules = [
        &kampus_events::MODULE,
        &kampus_access::MODULE,
        &kampus_identity::MODULE,
        &kampus_customise::MODULE,
    ];
    let schemas: Vec<_> = modules.iter().map(|m| m.schema()).collect();
    Upgrader::new(&db, &schemas, "0.1.0").apply(true).await.unwrap();
    let access = Access::new(db.clone(), Arc::new(SystemClock));
    access
        .sync_modules(&[&kampus_access::MODULE, &kampus_customise::MODULE])
        .await
        .unwrap();
    kampus_events::Events::new(db.clone())
        .sync_declarations(&[&kampus_customise::MODULE])
        .await
        .unwrap();
    let customise = Customise::new(db.clone(), access.clone());
    customise.sync_declarations(&[&TESTING]).await.unwrap();
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let category = access.create_scope(campus, ScopeKind::Category).await.unwrap();
    let course = access.create_scope(category, ScopeKind::Course).await.unwrap();
    let section = access.create_scope(course, ScopeKind::Section).await.unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();
    let admin = PrincipalId::generate();
    access.create_principal(admin, campus).await.unwrap();
    access
        .grant_for_module("setup", roles["campus_administrator"], campus, admin, None)
        .await
        .unwrap();
    let nobody = PrincipalId::generate();
    access.create_principal(nobody, campus).await.unwrap();
    World {
        customise,
        db,
        campus,
        category,
        course,
        section,
        admin: Actor::Principal(admin),
        nobody: Actor::Principal(nobody),
    }
}

impl World {
    /// Drops the installation.
    pub(crate) async fn drop(self) {
        for module in ["customise", "identity", "access", "events", "db"] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
    }

    /// A field of a type, defined in the campus.
    pub(crate) async fn field(
        &self,
        code: &str,
        field_type: FieldType,
        choices: &[&str],
    ) -> kampus_customise::Field {
        self.customise
            .create_field(
                &self.admin,
                "testing.thing",
                self.campus,
                FieldSpec {
                    code: code.to_owned(),
                    name: code.to_owned(),
                    field_type,
                    required: false,
                    visibility: FieldVisibility::Public,
                    max_length: Some(5),
                    choices: choices
                        .iter()
                        .map(|c| Choice {
                            code: (*c).to_owned(),
                            label: c.to_uppercase(),
                        })
                        .collect(),
                },
            )
            .await
            .unwrap()
    }

    /// Writes a value of a field in a transaction of its own and reads it back.
    pub(crate) async fn write(
        &self,
        field: &kampus_customise::Field,
        value: &Value,
    ) -> kampus_common::Result<Value> {
        let mut tx = self.db.begin().await.unwrap();
        let id = self.customise.write_value_in(&mut tx, field, None, value).await?;
        tx.commit().await.unwrap();
        let Some(id) = id else { return Ok(Value::Null) };
        Ok(self.customise.values(&[id]).await.unwrap().remove(&id).unwrap())
    }
}
