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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of activities, against PostgreSQL and a local store: course modules added in
//! order with the settings their type keeps, who sees them, changes with versions, moves, the recycle bin
//! and its purge, materials with their files, copies, and external references.

use std::collections::BTreeMap;
use std::sync::Arc;

use bytes::Bytes;
use kampus_access::{Access, Actor, ScopeId, ScopeKind};
use kampus_activities::{Activities, Module, ModuleFields, ModuleState};
use kampus_common::{AdapterSettings, ErrorKind, SystemClock};
use kampus_courses::{
    CampusMembers, CategoryFields, Course, CourseFields, Courses, Place, Section, Visibility,
};
use kampus_db::references::ExternalReference;
use kampus_db::{Database, Prefix, Upgrader};
use kampus_files::Files;
use kampus_identity::{FixedPolicies, Identity, MemoryCourier, Notice, PasswordCost, SecretKey};
use kampus_module::ServiceContext;
use serde_json::json;

/// The password of every account of these tests.
const PASSWORD: &str = "a-long-and-quiet-river";

/// The modules whose areas and types these tests use.
static MODULES: &[&kampus_module::Module] = &[&kampus_activities::MODULE, &crate::exceptions::QUIZ_MODULE];

/// A campus with a course of two sections, its administrator and a learner.
pub(crate) struct World {
    /// The service.
    pub(crate) activities: Activities,
    /// Courses.
    pub(crate) courses: Courses,
    /// Files.
    pub(crate) files: Files,
    /// The services, for the readers of files.
    pub(crate) services: ServiceContext,
    /// The database.
    pub(crate) db: Database,
    /// The course.
    pub(crate) course: Course,
    /// Its first section.
    pub(crate) first: Section,
    /// Its second section.
    pub(crate) second: Section,
    /// The administrator.
    pub(crate) admin: Actor,
    /// Someone who may only read.
    pub(crate) learner: Actor,
    /// Keeps the store.
    _store: tempfile::TempDir,
}

/// The fields of a page.
fn page(title: &str, text: &str) -> ModuleFields {
    ModuleFields {
        title: title.to_owned(),
        on_course_page: true,
        visibility: Visibility::Shown,
        settings: BTreeMap::from([("content".to_owned(), json!(text))]),
    }
}

/// The fields of a file material.
fn material(title: &str) -> ModuleFields {
    ModuleFields {
        title: title.to_owned(),
        on_course_page: true,
        visibility: Visibility::Shown,
        settings: BTreeMap::new(),
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
        Activities::new(db.clone(), access.clone(), courses.clone(), files.clone()).with_modules(MODULES);
    activities
        .sync_declarations(&[&kampus_activities::MODULE, &crate::exceptions::QUIZ_MODULE])
        .await
        .unwrap();
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
    let invited = identity
        .invite(&admin, campus, "lea@example.org", None)
        .await
        .unwrap();
    let Some(Notice::Invitation { token }) = courier.last_to("lea@example.org").map(|m| m.notice) else {
        panic!("an invitation was sent");
    };
    identity.accept_invitation(&token, PASSWORD).await.unwrap();
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
    let first = courses.sections(&admin, course.id).await.unwrap().remove(0);
    let second = courses
        .add_section(
            &admin,
            course.id,
            None,
            kampus_courses::SectionFields {
                title: Some("Unit 2".to_owned()),
                summary: None,
                visibility: Visibility::Shown,
            },
            Place::Last,
        )
        .await
        .unwrap();
    let services = ServiceContext::default().with(activities.clone());
    World {
        activities,
        courses,
        files,
        services,
        db,
        course,
        first,
        second,
        admin,
        learner: Actor::Principal(invited),
        _store: store,
    }
}

impl World {
    /// Drops the installation.
    pub(crate) async fn drop(self) {
        for module in [
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
        self.db.close().await;
    }

    /// Adds a page at the end of a section.
    pub(crate) async fn add_page(&self, section: &Section, title: &str) -> Module {
        self.activities
            .create_module(
                &self.admin,
                section.id,
                "activities.page",
                page(title, "words"),
                Place::Last,
            )
            .await
            .unwrap()
    }

    /// The titles of the course modules a person sees in a section.
    async fn titles(&self, who: &Actor, section: &Section) -> Vec<String> {
        self.activities
            .modules(who, section.id)
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.title)
            .collect()
    }

    /// Uploads a file into the materials area.
    async fn upload(&self, content: &'static [u8]) -> kampus_files::File {
        let size = i64::try_from(content.len()).unwrap();
        let campus = self.course.campus;
        let upload = self
            .files
            .open_upload(
                &self.admin,
                ("activities.material", campus),
                "notes.txt",
                size,
                None,
            )
            .await
            .unwrap();
        self.files
            .send_upload(&self.admin, upload.id, Bytes::from_static(content))
            .await
            .unwrap();
        self.files.complete_upload(&self.admin, upload.id).await.unwrap()
    }
}

/// The principal of an actor.
fn who(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) => Some(p.as_uuid()),
        _ => None,
    }
}

#[tokio::test]
async fn course_modules_are_added_in_order_with_the_settings_their_type_keeps() {
    let w = world().await;
    let a = w.add_page(&w.first, "A").await;
    let c = w.add_page(&w.first, "C").await;
    let b = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            "activities.page",
            page("B", "x"),
            Place::After(a.id),
        )
        .await
        .unwrap();
    let top = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            "activities.page",
            page("Top", "x"),
            Place::First,
        )
        .await
        .unwrap();
    assert_eq!(w.titles(&w.admin, &w.first).await, ["Top", "A", "B", "C"]);
    assert_eq!(b.settings["content"], json!("x"));
    assert_eq!(
        (c.type_code.as_str(), top.state),
        ("activities.page", ModuleState::Active)
    );
    let empty = w
        .activities
        .create_module(
            &w.admin,
            w.second.id,
            "activities.page",
            material("Empty"),
            Place::Last,
        )
        .await
        .unwrap();
    assert_eq!(
        empty.settings["content"],
        json!(""),
        "defaults fill what is left out"
    );
    let mut wrong = page("Wrong", "x");
    wrong.settings.insert("content".to_owned(), json!(5));
    wrong.settings.insert("colour".to_owned(), json!("red"));
    let invalid = w
        .activities
        .create_module(&w.admin, w.first.id, "activities.page", wrong, Place::Last)
        .await
        .unwrap_err();
    assert_eq!(invalid.code(), "activities.module_invalid");
    assert_eq!(invalid.fields().len(), 2);
    let blank = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            "activities.page",
            page(" ", "x"),
            Place::Last,
        )
        .await
        .unwrap_err();
    assert_eq!(blank.code(), "activities.module_invalid");
    let unknown = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            "activities.quiz",
            page("Q", "x"),
            Place::Last,
        )
        .await
        .unwrap_err();
    assert_eq!(unknown.code(), "activities.type_unknown");
    let denied = w
        .activities
        .create_module(
            &w.learner,
            w.first.id,
            "activities.page",
            page("Mine", "x"),
            Place::Last,
        )
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    assert_eq!(
        w.activities.types().await.unwrap().len(),
        3,
        "the two of activities and the quiz"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_hidden_course_module_is_seen_only_by_those_who_see_hidden_content() {
    let w = world().await;
    let shown = w.add_page(&w.first, "Shown").await;
    let mut hidden = page("Hidden", "x");
    hidden.visibility = Visibility::Hidden;
    let secret = w
        .activities
        .create_module(&w.admin, w.first.id, "activities.page", hidden, Place::Last)
        .await
        .unwrap();
    assert_eq!(w.titles(&w.learner, &w.first).await, ["Shown"]);
    assert_eq!(w.titles(&w.admin, &w.first).await, ["Shown", "Hidden"]);
    assert_eq!(
        w.activities
            .module(&w.learner, secret.id)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    w.activities.module(&w.learner, shown.id).await.unwrap();
    let mut hide_section = kampus_courses::SectionFields {
        title: Some("Unit 2".to_owned()),
        summary: None,
        visibility: Visibility::Hidden,
    };
    w.courses
        .update_section(&w.admin, w.second.id, hide_section.clone(), None)
        .await
        .unwrap();
    let inside = w.add_page(&w.second, "Inside").await;
    assert_eq!(
        w.activities
            .module(&w.learner, inside.id)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    hide_section.visibility = Visibility::Shown;
    w.drop().await;
}

#[tokio::test]
async fn a_course_module_changes_on_its_version_and_moves_with_its_scope() {
    let w = world().await;
    let a = w.add_page(&w.first, "A").await;
    let b = w.add_page(&w.first, "B").await;
    let changed = w
        .activities
        .update_module(&w.admin, a.id, page("A2", "new"), Some(a.version))
        .await
        .unwrap();
    assert_eq!((changed.title.as_str(), changed.version), ("A2", a.version + 1));
    let stale = w
        .activities
        .update_module(&w.admin, a.id, page("A3", "x"), Some(a.version))
        .await
        .unwrap_err();
    assert_eq!(stale.kind(), ErrorKind::PreconditionFailed);
    let moved = w
        .activities
        .move_module(&w.admin, b.id, w.second.id, Place::Last)
        .await
        .unwrap();
    assert_eq!(moved.section, w.second.id);
    assert_eq!(w.titles(&w.admin, &w.first).await, ["A2"]);
    assert_eq!(w.titles(&w.admin, &w.second).await, ["B"]);
    let mut conn = w.db.pool().acquire().await.unwrap();
    let access = Access::new(w.db.clone(), Arc::new(SystemClock));
    let path = access.path(&mut conn, moved.scope).await.unwrap();
    assert!(
        path.iter().any(|s| s.id == w.second.scope),
        "its scope moved under the new section"
    );
    assert!(!path.iter().any(|s| s.id == w.first.scope));
    drop(conn);
    let other = w
        .courses
        .create_course(
            &w.admin,
            w.course.category,
            CourseFields {
                short_name: "CHE101".to_owned(),
                full_name: "Chemistry".to_owned(),
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
    let foreign = w.courses.sections(&w.admin, other.id).await.unwrap().remove(0);
    let refused = w
        .activities
        .move_module(&w.admin, a.id, foreign.id, Place::Last)
        .await
        .unwrap_err();
    assert_eq!(refused.code(), "activities.section_of_another_course");
    let denied = w
        .activities
        .update_module(&w.learner, a.id, page("X", "x"), None)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn a_deleted_course_module_waits_in_the_bin_and_comes_back_or_is_purged() {
    let w = world().await;
    let a = w.add_page(&w.first, "A").await;
    let b = w.add_page(&w.first, "B").await;
    w.activities.delete_module(&w.admin, a.id).await.unwrap();
    assert_eq!(w.titles(&w.admin, &w.first).await, ["B"]);
    assert_eq!(
        w.activities.module(&w.admin, a.id).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    let bin = w.activities.bin(&w.admin, w.course.id).await.unwrap();
    assert_eq!(
        (bin.len(), bin[0].title.as_str(), bin[0].error.clone()),
        (1, "A", None)
    );
    assert_eq!(
        w.activities
            .delete_module(&w.admin, a.id)
            .await
            .unwrap_err()
            .code(),
        "activities.module_in_bin"
    );
    let restored = w.activities.restore_module(&w.admin, a.id).await.unwrap();
    assert_eq!(restored.sort_key, a.sort_key, "back in its place");
    assert_eq!(w.titles(&w.admin, &w.first).await, ["A", "B"]);
    assert_eq!(
        w.activities
            .restore_module(&w.admin, a.id)
            .await
            .unwrap_err()
            .code(),
        "activities.module_not_in_bin"
    );
    w.activities.delete_module(&w.admin, b.id).await.unwrap();
    let schema = w.db.prefix().schema("activities");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {schema}.bin_item set expires_at = now() - interval '1 hour'"
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    let purged = w.activities.purge().await.unwrap();
    assert_eq!((purged.removed, purged.failed), (1, 0));
    assert_eq!(w.activities.bin(&w.admin, w.course.id).await.unwrap().len(), 0);
    let gone: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {schema}.course_module where id = $1"
    )))
    .bind(b.id.as_uuid())
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(gone, 0);
    w.activities.delete_module(&w.admin, a.id).await.unwrap();
    w.courses
        .delete_section(&w.admin, w.first.id, None)
        .await
        .unwrap();
    assert_eq!(
        w.activities
            .restore_module(&w.admin, a.id)
            .await
            .unwrap_err()
            .code(),
        "activities.section_gone"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_file_material_is_read_by_who_sees_it_and_copied_with_its_file() {
    let w = world().await;
    let file = w.upload(b"lecture notes").await;
    let notes = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            "activities.file",
            material("Notes"),
            Place::Last,
        )
        .await
        .unwrap();
    let before = w
        .files
        .file(who(&w.learner), file.id, &w.services)
        .await
        .unwrap_err();
    assert_eq!(
        before.kind(),
        ErrorKind::NotFound,
        "nobody but the uploader reads it before it is attached"
    );
    w.activities
        .set_module_file(&w.admin, notes.id, file.id)
        .await
        .unwrap();
    w.files.file(who(&w.learner), file.id, &w.services).await.unwrap();
    assert_eq!(
        w.activities.module_file(&w.learner, notes.id).await.unwrap(),
        Some(file.id)
    );
    let page_module = w.add_page(&w.first, "Page").await;
    let not_file = w
        .activities
        .set_module_file(&w.admin, page_module.id, file.id)
        .await
        .unwrap_err();
    assert_eq!(not_file.code(), "activities.not_a_file_material");
    let mut hidden = material("Notes");
    hidden.visibility = Visibility::Hidden;
    w.activities
        .update_module(&w.admin, notes.id, hidden, None)
        .await
        .unwrap();
    assert_eq!(
        w.files
            .file(who(&w.learner), file.id, &w.services)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound,
        "hiding the material hides its file"
    );
    let copy = w
        .activities
        .copy_module(&w.admin, notes.id, w.second.id, Place::Last)
        .await
        .unwrap();
    let copied = w
        .activities
        .module_file(&w.admin, copy.id)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(copied, file.id);
    let schema = w.db.prefix().schema("files");
    let uses: i32 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select ref_count from {schema}.blob"
    )))
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(uses, 2, "the copy shares the content");
    w.activities.delete_module(&w.admin, copy.id).await.unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {}.bin_item set expires_at = now() - interval '1 hour'",
        w.db.prefix().schema("activities")
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    w.activities.purge().await.unwrap();
    let left: i32 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select ref_count from {schema}.blob"
    )))
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(left, 1, "purging the copy releases its use of the content");
    w.drop().await;
}

#[tokio::test]
async fn course_modules_are_found_and_updated_by_the_identifier_a_system_gives_them() {
    let w = world().await;
    let sis = |id: &str| ExternalReference::new("sis", id).unwrap();
    let (created, was_created) = w
        .activities
        .upsert_module_by_reference(
            &w.admin,
            w.first.id,
            "activities.page",
            &sis("M-1"),
            page("Intro", "a"),
        )
        .await
        .unwrap();
    assert!(was_created);
    let (again, was_created) = w
        .activities
        .upsert_module_by_reference(
            &w.admin,
            w.first.id,
            "activities.page",
            &sis("M-1"),
            page("Intro v2", "b"),
        )
        .await
        .unwrap();
    assert_eq!(
        (again.id, again.title.as_str(), was_created),
        (created.id, "Intro v2", false)
    );
    let campus: ScopeId = w.course.campus;
    assert_eq!(
        w.activities
            .module_by_reference(campus, &sis("M-1"))
            .await
            .unwrap(),
        Some(created.id)
    );
    let other = w.add_page(&w.first, "Other").await;
    let taken = w
        .activities
        .set_module_reference(&w.admin, other.id, &sis("M-1"))
        .await
        .unwrap_err();
    assert_eq!(taken.code(), "references.taken");
    w.activities
        .set_module_reference(&w.admin, other.id, &sis("M-2"))
        .await
        .unwrap();
    w.activities
        .remove_module_reference(&w.admin, other.id, "sis")
        .await
        .unwrap();
    assert_eq!(
        w.activities
            .module_external_references(&w.admin, other.id)
            .await
            .unwrap(),
        []
    );
    let denied = w
        .activities
        .set_module_reference(&w.learner, other.id, &sis("M-3"))
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}
