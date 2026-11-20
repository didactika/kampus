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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The installation the tests of records run in: a campus with a course, a teacher and two learners.

use std::sync::Arc;

use kampus_access::{Access, Actor, ScopeKind};
use kampus_activities::{Activities, Module, ModuleFields};
use kampus_common::{AdapterSettings, SystemClock};
use kampus_courses::{CampusMembers, CategoryFields, Course, CourseFields, Courses, Place, Visibility};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_files::Files;
use kampus_identity::{FixedPolicies, Identity, MemoryCourier, Notice, PasswordCost, SecretKey};
use kampus_records::Records;
use serde_json::json;

/// The password of every account of these tests.
const PASSWORD: &str = "a long and quiet river";

/// The modules whose types these tests use.
static MODULES: &[&kampus_module::Module] = &[&kampus_activities::MODULE];

/// A campus with a course, its administrator and two learners.
pub(crate) struct World {
    /// The service.
    pub(crate) records: Records,
    /// Courses, to find the sections of the course.
    pub(crate) courses: Courses,
    /// Activities, to place the activities that give grades.
    pub(crate) activities: Activities,
    /// The database.
    pub(crate) db: Database,
    /// The course.
    pub(crate) course: Course,
    /// The administrator, who also teaches.
    pub(crate) admin: Actor,
    /// A learner.
    pub(crate) lea: Actor,
    /// Another learner.
    pub(crate) max: Actor,
    /// Keeps the store.
    _store: tempfile::TempDir,
}

/// The principal of an actor.
pub(crate) fn who(actor: &Actor) -> kampus_access::PrincipalId {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => p.to_owned(),
        Actor::Visitor(_) => panic!("a person"),
    }
}

/// Builds the world.
#[allow(clippy::too_many_lines, reason = "one installation written top to bottom")]
pub(crate) async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("v{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let modules = [
        &kampus_events::MODULE,
        &kampus_tasks::MODULE,
        &kampus_access::MODULE,
        &kampus_identity::MODULE,
        &kampus_settings::MODULE,
        &kampus_customise::MODULE,
        &kampus_courses::MODULE,
        &kampus_files::MODULE,
        &kampus_activities::MODULE,
        &kampus_records::MODULE,
    ];
    let schemas: Vec<_> = modules.iter().map(|m| m.schema()).collect();
    Upgrader::new(&db, &schemas, "0.1.0").apply(true).await.unwrap();
    let clock = Arc::new(SystemClock);
    let access = Access::new(db.clone(), clock.clone());
    access.sync_modules(&modules).await.unwrap();
    kampus_events::Events::new(db.clone())
        .sync_declarations(&modules)
        .await
        .unwrap();
    let store = tempfile::tempdir().unwrap();
    let storage = kampus_storage::open(
        &[kampus_storage_opendal::ADAPTER],
        AdapterSettings {
            url: format!("file://{}", store.path().to_str().unwrap()),
            ..AdapterSettings::default()
        },
    )
    .await
    .unwrap();
    let files = Files::new(
        db.clone(),
        access.clone(),
        Some(storage),
        [3; 32],
        "http://kampus.test/api/v1".into(),
    )
    .with_modules(MODULES);
    files.sync_declarations(&modules).await.unwrap();
    let courses = Courses::new(db.clone(), access.clone(), clock.clone(), Arc::new(CampusMembers));
    let activities =
        Activities::new(db.clone(), access.clone(), courses.clone(), files).with_modules(MODULES);
    activities.sync_declarations(&modules).await.unwrap();
    let records = Records::new(db.clone(), access.clone(), courses.clone(), activities.clone());
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    access.install_campus_roles(campus).await.unwrap();
    let courier = Arc::new(MemoryCourier::default());
    let identity = Identity::new(
        db.clone(),
        access.clone(),
        clock,
        Arc::new(FixedPolicies::default()),
        courier.clone(),
        SecretKey::generate().unwrap(),
    )
    .with_password_cost(PasswordCost {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    });
    let admin = Actor::Principal(
        identity
            .install_administrator(campus, "admin@example.org", "admin", PASSWORD)
            .await
            .unwrap(),
    );
    let mut learners = Vec::new();
    for email in ["lea@example.org", "max@example.org"] {
        let invited = identity.invite(&admin, campus, email, None).await.unwrap();
        let Some(Notice::Invitation { token }) = courier.last_to(email).map(|m| m.notice) else {
            panic!("an invitation was sent");
        };
        identity.accept_invitation(&token, PASSWORD).await.unwrap();
        learners.push(Actor::Principal(invited));
    }
    let category = courses
        .create_category(
            &admin,
            campus,
            None,
            CategoryFields {
                name: "Science".to_owned(),
                code: None,
                description: None,
                visibility: Visibility::Shown,
            },
            Place::Last,
        )
        .await
        .unwrap();
    let course = courses
        .create_course(
            &admin,
            category.id,
            CourseFields {
                short_name: "PHY101".to_owned(),
                full_name: "Physics I".to_owned(),
                code: None,
                summary: None,
                layout: None,
                language_code: None,
                starts_at: None,
                ends_at: None,
                visibility: Visibility::Shown,
            },
            Place::Last,
        )
        .await
        .unwrap();
    let max = learners.pop().unwrap();
    let lea = learners.pop().unwrap();
    let learner_role = {
        let mut conn = db.pool().acquire().await.unwrap();
        access.campus_role_in(&mut conn, campus, "learner").await.unwrap()
    };
    for learner in [&lea, &max] {
        access
            .grant_for_module("enrol", learner_role, course.scope, who(learner), None)
            .await
            .unwrap();
    }
    World {
        records,
        courses,
        activities,
        db,
        course,
        admin,
        lea,
        max,
        _store: store,
    }
}

impl World {
    /// Adds a page to the first section of the course, as the activity that gives grades in tests.
    pub(crate) async fn page(&self, title: &str) -> Module {
        let section = self
            .courses
            .sections(&self.admin, self.course.id)
            .await
            .unwrap()
            .remove(0);
        self.activities
            .create_module(
                &self.admin,
                section.id,
                "activities.page",
                ModuleFields {
                    title: title.to_owned(),
                    on_course_page: true,
                    visibility: Visibility::Shown,
                    settings: std::collections::BTreeMap::from([("content".to_owned(), json!("words"))]),
                },
                Place::Last,
            )
            .await
            .unwrap()
    }

    /// Drops the installation.
    pub(crate) async fn drop(self) {
        for module in [
            "records",
            "activities",
            "files",
            "courses",
            "customise",
            "settings",
            "identity",
            "access",
            "tasks",
            "events",
            "db",
        ] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
    }
}
