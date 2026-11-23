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

use std::sync::{Arc, Mutex};

use kampus_access::{Access, Actor, PrincipalId, ScopeId, ScopeKind};
use kampus_common::SystemClock;
use kampus_customise::{Choice, Customise, FieldSpec, FieldType, FieldVisibility};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_module::{
    Cells, EventDeclaration, FieldKindDeclaration, FlagDeclaration, Grid, JobFuture, MenuKindDeclaration,
    Module, PageKindDeclaration, Resize, RuleActionDeclaration, RuleActionRun, RuleParameter,
    TermDeclaration, TermForms, ValueKind, WidgetDeclaration, WidgetSetting, WidgetSize,
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
const PAGES: &[PageKindDeclaration] = &[
    PageKindDeclaration {
        code: "testing.page",
        summary: "A page",
        grid: Grid::STANDARD,
        personal: false,
    },
    PageKindDeclaration {
        code: "testing.lesson",
        summary: "A page that always holds its anchor",
        grid: Grid::STANDARD,
        personal: false,
    },
    PageKindDeclaration {
        code: "testing.home",
        summary: "A page each person arranges for themselves",
        grid: Grid::STANDARD,
        personal: true,
    },
];

/// The sizes of the clock on a page kind: two by two up to six by four.
const fn clock(page_kind: &'static str) -> WidgetSize {
    WidgetSize {
        page_kind,
        min: Cells::new(2, 2),
        default: Cells::new(3, 2),
        max: Cells::new(6, 4),
        resize: Resize::Both,
        bands: &[],
    }
}

/// The widgets of these tests.
const WIDGETS: &[WidgetDeclaration] = &[
    WidgetDeclaration {
        code: "testing.clock",
        summary: "A clock",
        icon: "schedule",
        single: false,
        required: false,
        sizes: &[
            clock("testing.page"),
            clock("testing.home"),
            clock("testing.lesson"),
        ],
        settings: &[WidgetSetting {
            key: "seconds",
            kind: ValueKind::Boolean { default: false },
        }],
    },
    WidgetDeclaration {
        code: "testing.elsewhere",
        summary: "A widget for another page",
        icon: "widgets",
        single: false,
        required: false,
        sizes: &[clock("testing.other")],
        settings: &[],
    },
    WidgetDeclaration {
        code: "testing.anchor",
        summary: "What every lesson holds",
        icon: "anchor",
        single: true,
        required: true,
        sizes: &[WidgetSize::fixed("testing.lesson", Cells::new(8, 4))],
        settings: &[],
    },
    WidgetDeclaration {
        code: "testing.banner",
        summary: "A banner a page holds once",
        icon: "flag",
        single: true,
        required: false,
        sizes: &[clock("testing.page")],
        settings: &[],
    },
];

/// The kinds of menu of these tests.
const MENUS: &[MenuKindDeclaration] = &[MenuKindDeclaration {
    code: "testing.menu",
    summary: "A menu",
}];

/// The event types of these tests.
const EVENTS: &[EventDeclaration] = &[EventDeclaration {
    code: "testing.happened",
    version: 1,
    summary: "Something happened",
    fields: &["account", "level"],
    audited: false,
    personal_fields: &[],
}];

/// What the action of these tests noted: the event and the parameters, by event.
pub(crate) static NOTED: Mutex<Vec<(uuid::Uuid, Value)>> = Mutex::new(Vec::new());

/// Notes the event with its parameters.
fn note(run: RuleActionRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        NOTED.lock().unwrap().push((run.event.id, run.params.clone()));
        Ok(())
    })
}

/// Fails.
fn fail(_: RuleActionRun<'_>) -> JobFuture<'_> {
    Box::pin(async { Err(kampus_common::Error::conflict("testing.failed")) })
}

/// The rule actions of these tests.
const ACTIONS: &[RuleActionDeclaration] = &[
    RuleActionDeclaration {
        code: "testing.note",
        summary: "Notes the event",
        params: &[
            RuleParameter {
                key: "tag",
                kind: ValueKind::Text { max: 20, default: "" },
                required: true,
            },
            RuleParameter {
                key: "loud",
                kind: ValueKind::Boolean { default: false },
                required: false,
            },
        ],
        needs: &["account"],
        permission: "customise.flag.manage",
        publishes: &[],
        run: note,
    },
    RuleActionDeclaration {
        code: "testing.fail",
        summary: "Fails",
        params: &[],
        needs: &[],
        permission: "customise.flag.manage",
        publishes: &[],
        run: fail,
    },
    RuleActionDeclaration {
        code: "testing.echo",
        summary: "Publishes what triggers it",
        params: &[],
        needs: &[],
        permission: "customise.flag.manage",
        publishes: &["testing.happened"],
        run: fail,
    },
];
/// A module that declares what these tests customise.
pub(crate) const TESTING: Module = Module {
    name: "testing",
    terms: TERMS,
    flags: FLAGS,
    field_kinds: KINDS,
    page_kinds: PAGES,
    widgets: WIDGETS,
    menu_kinds: MENUS,
    events: EVENTS,
    listeners: &[],
    rule_actions: ACTIONS,
    ..kampus_customise::MODULE
};

/// The modules whose rule actions the tests run.
static MODULES: &[&Module] = &[&TESTING];

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
    /// A person of the campus, with their own scope.
    pub(crate) person: Actor,
    /// The scope of the person.
    pub(crate) person_scope: ScopeId,
    /// The scope of another person.
    pub(crate) other_scope: ScopeId,
}

/// Builds the world in an installation of its own prefix.
/// One scope, owned by one person.
#[derive(Debug)]
struct OneOwner {
    scope: ScopeId,
    owner: PrincipalId,
}

#[async_trait::async_trait]
impl kampus_customise::ScopeOwners for OneOwner {
    async fn owner_of(&self, scope: ScopeId) -> kampus_common::Result<Option<PrincipalId>> {
        Ok((scope == self.scope).then_some(self.owner))
    }
}

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
        .sync_declarations(&[&kampus_events::MODULE, &kampus_customise::MODULE, &TESTING])
        .await
        .unwrap();
    let customise = Customise::new(db.clone(), access.clone()).with_modules(MODULES);
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
    let person = PrincipalId::generate();
    access.create_principal(person, campus).await.unwrap();
    access
        .grant_for_module("setup", roles["authenticated_user"], campus, person, None)
        .await
        .unwrap();
    let person_scope = access.create_scope(campus, ScopeKind::User).await.unwrap();
    let other_scope = access.create_scope(campus, ScopeKind::User).await.unwrap();
    let customise = customise.with_owners(Arc::new(OneOwner {
        scope: person_scope,
        owner: person,
    }));
    World {
        customise,
        db,
        campus,
        category,
        course,
        section,
        admin: Actor::Principal(admin),
        nobody: Actor::Principal(nobody),
        person: Actor::Principal(person),
        person_scope,
        other_scope,
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
        self.db.close().await;
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
