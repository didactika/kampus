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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of courses, against PostgreSQL: the category tree, courses and their
//! lifecycle, nested sections, effective visibility, and groups with their members and sets.

use std::sync::Arc;

use kampus_access::{Access, Actor, RoleId, ScopeId, ScopeKind};
use kampus_common::{ErrorKind, Result, SystemClock, Timestamp};
use kampus_courses::{
    CampusMembers, CategoryFields, CourseFields, CourseState, Courses, GroupFields, GroupMode,
    GroupVisibility, MemberOrigin, Place, SectionFields, Visibility,
};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_identity::{AccountId, FixedPolicies, Identity, MemoryCourier, Notice, PasswordCost, SecretKey};

/// The password of every account of these tests.
const PASSWORD: &str = "a long and quiet river";

/// A campus with its administrator, its roles and people.
struct World {
    /// The service.
    courses: Courses,
    /// Access.
    access: Access,
    /// Identity, to create people.
    identity: Identity,
    /// The messages identity sent.
    courier: Arc<MemoryCourier>,
    /// The database.
    db: Database,
    /// The campus.
    campus: ScopeId,
    /// The roles of the campus, by template.
    roles: std::collections::BTreeMap<&'static str, RoleId>,
    /// The campus administrator.
    admin: Actor,
}

/// The error kind and code of a result.
fn error<T: std::fmt::Debug>(result: Result<T>) -> (ErrorKind, String) {
    let error = result.expect_err("the operation fails");
    (error.kind(), error.code().to_owned())
}

/// Fields of a category.
fn category(name: &str) -> CategoryFields {
    CategoryFields {
        name: name.to_owned(),
        code: None,
        description: None,
        visibility: Visibility::Shown,
    }
}

/// Fields of a course.
fn course(short_name: &str) -> CourseFields {
    CourseFields {
        short_name: short_name.to_owned(),
        full_name: format!("{short_name}, the course"),
        code: None,
        summary: None,
        layout: None,
        language_code: None,
        starts_at: None,
        ends_at: None,
        visibility: Visibility::Shown,
    }
}

/// Fields of a section.
fn section(title: &str) -> SectionFields {
    SectionFields {
        title: Some(title.to_owned()),
        summary: None,
        visibility: Visibility::Shown,
    }
}

/// Fields of a group.
fn group(name: &str, visibility: GroupVisibility) -> GroupFields {
    GroupFields {
        name: name.to_owned(),
        code: None,
        description: None,
        section: None,
        visibility,
    }
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("c{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let modules = [
        &kampus_events::MODULE,
        &kampus_access::MODULE,
        &kampus_identity::MODULE,
        &kampus_settings::MODULE,
        &kampus_customise::MODULE,
        &kampus_courses::MODULE,
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
    kampus_settings::Settings::new(db.clone(), access.clone())
        .sync_declarations(&modules)
        .await
        .unwrap();
    let customise = kampus_customise::Customise::new(db.clone(), access.clone());
    customise.sync_declarations(&modules).await.unwrap();
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();
    let courier = Arc::new(MemoryCourier::default());
    let identity = Identity::new(
        db.clone(),
        access.clone(),
        clock.clone(),
        Arc::new(FixedPolicies::default()),
        courier.clone(),
        SecretKey::generate().unwrap(),
    )
    .with_password_cost(PasswordCost {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    });
    let admin = identity
        .install_administrator(campus, "admin@example.org", "admin", PASSWORD)
        .await
        .unwrap();
    let courses = Courses::new(db.clone(), access.clone(), clock, Arc::new(CampusMembers))
        .with_customise(customise.clone());
    World {
        courses,
        access,
        identity,
        courier,
        db,
        campus,
        roles,
        admin: Actor::Principal(admin),
    }
}

impl World {
    /// Invites a person to the campus and accepts the invitation.
    async fn person(&self, email: &str) -> Actor {
        let id: AccountId = self
            .identity
            .invite(&self.admin, self.campus, email, None)
            .await
            .unwrap();
        let Some(Notice::Invitation { token }) = self.courier.last_to(email).map(|m| m.notice) else {
            panic!("an invitation was sent");
        };
        self.identity.accept_invitation(&token, PASSWORD).await.unwrap();
        Actor::Principal(id)
    }

    /// Gives a person a role of the campus in a scope.
    async fn grant(&self, role: &str, scope: ScopeId, to: &Actor) {
        let Actor::Principal(to) = to else {
            panic!("a person");
        };
        self.access
            .grant(&self.admin, self.roles[role], scope, *to, None)
            .await
            .unwrap();
    }

    /// Drops the installation.
    async fn drop(self) {
        for module in [
            "courses",
            "customise",
            "settings",
            "identity",
            "access",
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

#[tokio::test]
async fn categories_form_a_tree_in_their_own_order_with_codes_unique_in_the_campus() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let arts = c
        .create_category(&w.admin, w.campus, None, category("Arts"), Place::First)
        .await
        .unwrap();
    let physics = c
        .create_category(
            &w.admin,
            w.campus,
            Some(science.id),
            category("Physics"),
            Place::Last,
        )
        .await
        .unwrap();
    let top: Vec<_> = c
        .categories(&w.admin, w.campus, None)
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(top, ["Arts", "Science"]);
    assert_eq!(physics.parent, Some(science.id));

    let coded = CategoryFields {
        code: Some("SCI".to_owned()),
        ..category("Sciences")
    };
    c.create_category(&w.admin, w.campus, None, coded.clone(), Place::Last)
        .await
        .unwrap();
    let lower = CategoryFields {
        code: Some("sci".to_owned()),
        ..coded
    };
    assert_eq!(
        error(
            c.create_category(&w.admin, w.campus, None, lower, Place::Last)
                .await
        ),
        (ErrorKind::Conflict, "courses.category_code_taken".to_owned()),
        "codes are unique whatever their case"
    );

    let moved = c
        .move_category(
            &w.admin,
            arts.id,
            None,
            Place::After(science.id),
            Some(arts.version),
        )
        .await
        .unwrap();
    assert_eq!(moved.version, arts.version + 1);
    let top: Vec<_> = c
        .categories(&w.admin, w.campus, None)
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(top[..2], ["Science", "Arts"]);
    assert_eq!(
        c.category(science.id).await.unwrap().sort_key,
        science.sort_key,
        "moving one category never touches its siblings"
    );
    assert_eq!(
        error(
            c.move_category(&w.admin, science.id, Some(physics.id), Place::Last, None)
                .await
        )
        .1,
        "access.scope_parent_invalid",
        "a category cannot go inside itself"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_category_is_deleted_only_when_empty() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    assert_eq!(
        error(c.delete_category(&w.admin, science.id, None).await).1,
        "courses.category_not_empty"
    );
    let empty = c
        .create_category(&w.admin, w.campus, None, category("Empty"), Place::Last)
        .await
        .unwrap();
    c.delete_category(&w.admin, empty.id, Some(empty.version))
        .await
        .unwrap();
    assert_eq!(error(c.category(empty.id).await).0, ErrorKind::NotFound);
    assert_eq!(c.course(physics.id).await.unwrap().category, science.id);
    w.drop().await;
}

#[tokio::test]
async fn a_new_course_is_a_draft_with_a_first_section_and_its_creator_owns_and_teaches_it() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    assert_eq!(
        error(
            c.create_course(&ana, science.id, course("PHY101"), Place::Last)
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
    w.grant("course_creator", science.scope, &ana).await;
    let physics = c
        .create_course(&ana, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    assert_eq!(physics.state, CourseState::Draft);
    assert_eq!(physics.layout, "topics");
    let Actor::Principal(ana_id) = ana else {
        unreachable!()
    };
    assert_eq!(c.owners(physics.id).await.unwrap(), [ana_id]);
    assert_eq!(c.sections(&ana, physics.id).await.unwrap().len(), 1);

    let renamed = c
        .update_course(
            &ana,
            physics.id,
            CourseFields {
                full_name: "Physics I".to_owned(),
                ..course("PHY101")
            },
            Some(physics.version),
        )
        .await
        .unwrap();
    assert_eq!(renamed.full_name, "Physics I");
    assert_eq!(
        error(
            c.update_course(&ana, physics.id, course("PHY101"), Some(physics.version))
                .await
        )
        .0,
        ErrorKind::PreconditionFailed,
        "a change made on an old version is refused"
    );
    assert_eq!(
        error(
            c.create_course(&w.admin, science.id, course("phy101"), Place::Last)
                .await
        )
        .1,
        "courses.course_name_taken",
        "short names are unique in the campus whatever their case"
    );
    let backwards = CourseFields {
        starts_at: Some(Timestamp::parse_rfc3339("2027-01-01T00:00:00Z").unwrap()),
        ends_at: Some(Timestamp::parse_rfc3339("2026-01-01T00:00:00Z").unwrap()),
        ..course("PHY102")
    };
    assert_eq!(
        error(
            c.create_course(&w.admin, science.id, backwards, Place::Last)
                .await
        )
        .1,
        "courses.course_invalid"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_course_goes_through_its_lifecycle_and_an_archived_one_is_read_only() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    w.grant("course_creator", science.scope, &ana).await;
    let physics = c
        .create_course(&ana, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    assert_eq!(
        error(
            c.transition_course(&ana, physics.id, CourseState::Archived, None)
                .await
        )
        .1,
        "courses.transition_invalid",
        "a draft is published before it is archived"
    );
    let published = c
        .transition_course(&ana, physics.id, CourseState::Published, None)
        .await
        .unwrap();
    let archived = c
        .transition_course(&ana, physics.id, CourseState::Archived, Some(published.version))
        .await
        .unwrap();
    assert_eq!(archived.state, CourseState::Archived);
    assert_eq!(
        error(c.update_course(&ana, physics.id, course("PHY101"), None).await).0,
        ErrorKind::PermissionDenied,
        "an archived course is read-only"
    );
    assert_eq!(
        error(
            c.add_section(&ana, physics.id, None, section("More"), Place::Last)
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
    c.transition_course(&ana, physics.id, CourseState::Published, None)
        .await
        .unwrap();
    c.update_course(&ana, physics.id, course("PHY101"), None)
        .await
        .unwrap();

    let scheduled = c
        .transition_course(&ana, physics.id, CourseState::DeletionScheduled, None)
        .await
        .unwrap();
    assert!(
        scheduled.deletion_due_at.is_some(),
        "its owner schedules the deletion"
    );
    let cancelled = c
        .transition_course(&ana, physics.id, CourseState::Draft, None)
        .await
        .unwrap();
    assert_eq!(cancelled.state, CourseState::Draft);
    assert!(cancelled.deletion_due_at.is_none());

    let bea = w.person("bea@example.org").await;
    w.grant("teacher", physics.scope, &bea).await;
    assert_eq!(
        error(
            c.transition_course(&bea, physics.id, CourseState::DeletionScheduled, None)
                .await
        )
        .0,
        ErrorKind::PermissionDenied,
        "a teacher who does not own the course cannot delete it"
    );
    w.drop().await;
}

#[tokio::test]
async fn hiding_a_category_hides_what_is_in_it_and_showing_it_restores_everything() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let hidden_course = c
        .create_course(
            &w.admin,
            science.id,
            CourseFields {
                visibility: Visibility::Hidden,
                ..course("PHY102")
            },
            Place::Last,
        )
        .await
        .unwrap();
    let learner = w.person("lea@example.org").await;
    let seen: Vec<_> = c
        .courses(&learner, science.id)
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(seen, [physics.id]);
    assert_eq!(c.courses(&w.admin, science.id).await.unwrap().len(), 2);

    let hidden = c
        .update_category(
            &w.admin,
            science.id,
            CategoryFields {
                visibility: Visibility::Hidden,
                ..category("Science")
            },
            None,
        )
        .await
        .unwrap();
    assert!(!c.shown(physics.scope).await.unwrap());
    assert_eq!(c.courses(&learner, science.id).await.unwrap(), []);
    assert!(!c.sees(&learner, physics.scope).await.unwrap());
    assert!(
        c.sees(&w.admin, physics.scope).await.unwrap(),
        "who may see hidden content sees it"
    );

    c.update_category(&w.admin, science.id, category("Science"), Some(hidden.version))
        .await
        .unwrap();
    assert!(c.shown(physics.scope).await.unwrap());
    assert!(
        !c.shown(hidden_course.scope).await.unwrap(),
        "a course hidden on its own stays hidden: nothing was copied down"
    );

    let past = Visibility::Scheduled {
        from: None,
        until: Some(Timestamp::parse_rfc3339("2020-01-01T00:00:00Z").unwrap()),
    };
    let ended = c
        .create_course(
            &w.admin,
            science.id,
            CourseFields {
                visibility: past,
                ..course("PHY103")
            },
            Place::Last,
        )
        .await
        .unwrap();
    assert!(
        !c.shown(ended.scope).await.unwrap(),
        "a window that ended hides the course"
    );
    w.drop().await;
}

#[tokio::test]
async fn sections_nest_move_with_their_subtree_and_come_back_from_deletion() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let first = c.sections(&w.admin, physics.id).await.unwrap()[0].clone();
    let one = c
        .add_section(&w.admin, physics.id, None, section("Unit 1"), Place::Last)
        .await
        .unwrap();
    let lesson = c
        .add_section(
            &w.admin,
            physics.id,
            Some(one.id),
            section("Lesson 1.1"),
            Place::Last,
        )
        .await
        .unwrap();
    let two = c
        .add_section(&w.admin, physics.id, None, section("Unit 2"), Place::Last)
        .await
        .unwrap();
    let titles = |sections: Vec<kampus_courses::Section>| -> Vec<String> {
        sections.into_iter().filter_map(|s| s.title).collect()
    };
    assert_eq!(
        titles(c.sections(&w.admin, physics.id).await.unwrap()),
        ["Unit 1", "Lesson 1.1", "Unit 2"],
        "depth first, in order"
    );

    let moved = c
        .move_section(&w.admin, one.id, Some(two.id), Place::Last, Some(one.version))
        .await
        .unwrap();
    assert_eq!(moved.parent, Some(two.id));
    assert_eq!(
        titles(c.sections(&w.admin, physics.id).await.unwrap()),
        ["Unit 2", "Unit 1", "Lesson 1.1"],
        "the subtree moved with it"
    );
    assert!(
        w.access
            .path(&mut w.db.pool().acquire().await.unwrap(), lesson.scope)
            .await
            .unwrap()
            .iter()
            .any(|s| s.id == two.scope),
        "the scope of a nested section follows its parent"
    );
    assert_eq!(
        error(
            c.move_section(&w.admin, two.id, Some(lesson.id), Place::Last, None)
                .await
        )
        .1,
        "access.scope_parent_invalid",
        "a section cannot go inside its own subtree"
    );
    assert_eq!(c.section(first.id).await.unwrap().sort_key, first.sort_key);

    c.delete_section(&w.admin, two.id, None).await.unwrap();
    assert_eq!(
        titles(c.sections(&w.admin, physics.id).await.unwrap()),
        Vec::<String>::new()
    );
    assert_eq!(
        error(c.restore_section(&w.admin, lesson.id).await).1,
        "courses.section_parent_deleted"
    );
    c.restore_section(&w.admin, two.id).await.unwrap();
    assert_eq!(
        titles(c.sections(&w.admin, physics.id).await.unwrap()),
        ["Unit 2", "Unit 1", "Lesson 1.1"],
        "the sections deleted with it come back"
    );
    w.drop().await;
}

#[tokio::test]
async fn groups_have_unique_names_and_each_origin_removes_only_its_memberships() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let red = c
        .create_group(&w.admin, physics.id, group("Red", GroupVisibility::Members))
        .await
        .unwrap();
    assert_eq!(
        error(
            c.create_group(&w.admin, physics.id, group("red", GroupVisibility::Everyone))
                .await
        )
        .1,
        "courses.group_name_taken"
    );
    let ana = w.person("ana@example.org").await;
    let Actor::Principal(ana_id) = ana else {
        unreachable!()
    };
    c.add_member(&w.admin, red.id, ana_id).await.unwrap();
    c.add_module_member("enrol", red.id, ana_id).await.unwrap();
    c.remove_member(&w.admin, red.id, ana_id).await.unwrap();
    let members = c.members(&w.admin, red.id).await.unwrap();
    assert_eq!(members.len(), 1, "the membership enrolment added stays");
    assert_eq!(members[0].origin, MemberOrigin::Module("enrol".to_owned()));
    c.remove_module_member("enrol", red.id, ana_id).await.unwrap();
    assert_eq!(c.members(&w.admin, red.id).await.unwrap(), []);
    assert_eq!(
        error(c.add_module_member("manual", red.id, ana_id).await).1,
        "courses.member_origin_invalid"
    );

    let secret = c
        .create_group(&w.admin, physics.id, group("Blue", GroupVisibility::OwnOnly))
        .await
        .unwrap();
    let bea = w.person("bea@example.org").await;
    let Actor::Principal(bea_id) = bea else {
        unreachable!()
    };
    c.add_member(&w.admin, secret.id, ana_id).await.unwrap();
    c.add_member(&w.admin, secret.id, bea_id).await.unwrap();
    let seen = c.members(&ana, secret.id).await.unwrap();
    assert_eq!(seen.len(), 1, "each member sees only themselves");
    assert_eq!(seen[0].account, ana_id);
    let groups: Vec<_> = c
        .groups(&bea, physics.id)
        .await
        .unwrap()
        .into_iter()
        .map(|g| g.name)
        .collect();
    assert_eq!(
        groups,
        ["Blue"],
        "a group shown to its members is not shown to others"
    );
    assert!(Courses::check_group_mode_group(&secret).is_err());
    assert!(Courses::check_group_mode_group(&red).is_ok());
    w.drop().await;
}

#[tokio::test]
async fn a_group_set_holds_groups_of_its_own_course_once() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let chemistry = c
        .create_course(&w.admin, science.id, course("CHE101"), Place::Last)
        .await
        .unwrap();
    let red = c
        .create_group(&w.admin, physics.id, group("Red", GroupVisibility::Everyone))
        .await
        .unwrap();
    let other = c
        .create_group(&w.admin, chemistry.id, group("Red", GroupVisibility::Everyone))
        .await
        .unwrap();
    let set = c
        .create_group_set(&w.admin, physics.id, "Labs", None)
        .await
        .unwrap();
    c.add_to_set(&w.admin, set.id, red.id).await.unwrap();
    c.add_to_set(&w.admin, set.id, red.id).await.unwrap();
    assert_eq!(
        error(c.add_to_set(&w.admin, set.id, other.id).await).1,
        "courses.group_set_group_invalid"
    );
    let sets = c.group_sets(&w.admin, physics.id).await.unwrap();
    assert_eq!(sets[0].groups, [red.id]);
    c.remove_from_set(&w.admin, set.id, red.id).await.unwrap();
    assert_eq!(c.group_sets(&w.admin, physics.id).await.unwrap()[0].groups, []);
    w.drop().await;
}

#[tokio::test]
async fn a_hidden_section_hides_the_sections_inside_it() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let unit = c
        .add_section(&w.admin, physics.id, None, section("Unit 1"), Place::Last)
        .await
        .unwrap();
    c.add_section(
        &w.admin,
        physics.id,
        Some(unit.id),
        section("Lesson 1.1"),
        Place::Last,
    )
    .await
    .unwrap();
    let hidden = c
        .update_section(
            &w.admin,
            unit.id,
            SectionFields {
                visibility: Visibility::Hidden,
                ..section("Unit 1")
            },
            Some(unit.version),
        )
        .await
        .unwrap();
    assert_eq!(hidden.version, unit.version + 1);
    let learner = w.person("lea@example.org").await;
    assert_eq!(
        c.sections(&learner, physics.id).await.unwrap().len(),
        1,
        "only the first section remains: the hidden one takes its lessons with it"
    );
    assert_eq!(c.sections(&w.admin, physics.id).await.unwrap().len(), 3);
    w.drop().await;
}

#[tokio::test]
async fn a_course_whose_deletion_is_due_goes_with_everything_it_owns() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let kept = c
        .create_course(&w.admin, science.id, course("PHY102"), Place::Last)
        .await
        .unwrap();
    c.create_group(&w.admin, physics.id, group("Red", GroupVisibility::Everyone))
        .await
        .unwrap();
    c.transition_course(&w.admin, physics.id, CourseState::DeletionScheduled, None)
        .await
        .unwrap();
    assert_eq!(
        c.run_due_deletions().await.unwrap(),
        0,
        "the grace period has not ended"
    );
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {}.\"course\" set deletion_due_at = now() - interval '1 minute' where id = '{}'",
        w.db.prefix().schema("courses"),
        physics.id
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    assert_eq!(c.run_due_deletions().await.unwrap(), 1);
    assert_eq!(error(c.course(physics.id).await).0, ErrorKind::NotFound);
    assert_eq!(
        error(
            w.access
                .path(&mut w.db.pool().acquire().await.unwrap(), physics.scope)
                .await
        )
        .0,
        ErrorKind::NotFound,
        "its scope went with it"
    );
    assert_eq!(c.course(kept.id).await.unwrap().short_name, "PHY102");
    w.drop().await;
}

#[tokio::test]
async fn the_group_mode_of_a_section_comes_from_its_course_unless_it_sets_its_own() {
    let w = world().await;
    let c = &w.courses;
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    let labs = c
        .add_section(&w.admin, physics.id, None, section("Labs"), Place::Last)
        .await
        .unwrap();
    assert_eq!(c.group_mode(labs.scope).await.unwrap(), GroupMode::None);
    let settings = kampus_settings::Settings::new(w.db.clone(), w.access.clone());
    settings
        .set(
            &w.admin,
            "courses.group_mode",
            physics.scope,
            serde_json::json!("visible"),
            false,
            None,
        )
        .await
        .unwrap();
    assert_eq!(c.group_mode(labs.scope).await.unwrap(), GroupMode::Visible);
    settings
        .set(
            &w.admin,
            "courses.group_mode",
            labs.scope,
            serde_json::json!("separate"),
            false,
            None,
        )
        .await
        .unwrap();
    assert_eq!(c.group_mode(labs.scope).await.unwrap(), GroupMode::Separate);
    assert_eq!(c.group_mode(physics.scope).await.unwrap(), GroupMode::Visible);
    w.drop().await;
}

#[tokio::test]
async fn a_course_keeps_values_of_the_custom_fields_that_apply_to_it() {
    let w = world().await;
    let c = &w.courses;
    let customise = kampus_customise::Customise::new(w.db.clone(), w.access.clone());
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    customise
        .create_field(
            &w.admin,
            "courses.course",
            science.scope,
            kampus_customise::FieldSpec {
                code: "credits".to_owned(),
                name: "Credits".to_owned(),
                field_type: kampus_customise::FieldType::Integer,
                required: false,
                visibility: kampus_customise::FieldVisibility::Public,
                max_length: None,
                choices: Vec::new(),
            },
        )
        .await
        .unwrap();
    let fields = c.course_fields(&w.admin, physics.id).await.unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].value, serde_json::Value::Null);
    let set = c
        .set_course_field(&w.admin, physics.id, "credits", &serde_json::json!(6))
        .await
        .unwrap();
    assert_eq!(set.value, serde_json::json!(6));
    c.set_course_field(&w.admin, physics.id, "credits", &serde_json::json!(8))
        .await
        .unwrap();
    assert_eq!(
        c.course_fields(&w.admin, physics.id).await.unwrap()[0].value,
        serde_json::json!(8)
    );
    assert_eq!(
        error(
            c.set_course_field(&w.admin, physics.id, "credits", &serde_json::json!("many"))
                .await
        )
        .0,
        ErrorKind::Validation
    );
    assert_eq!(
        error(
            c.set_course_field(&w.admin, physics.id, "nothing", &serde_json::json!(1))
                .await
        )
        .1,
        "courses.field_not_found"
    );
    c.set_course_field(&w.admin, physics.id, "credits", &serde_json::Value::Null)
        .await
        .unwrap();
    assert_eq!(
        c.course_fields(&w.admin, physics.id).await.unwrap()[0].value,
        serde_json::Value::Null
    );
    w.drop().await;
}

#[tokio::test]
async fn groups_cannot_be_created_where_the_feature_is_off() {
    let w = world().await;
    let c = &w.courses;
    let customise = kampus_customise::Customise::new(w.db.clone(), w.access.clone());
    let science = c
        .create_category(&w.admin, w.campus, None, category("Science"), Place::Last)
        .await
        .unwrap();
    let physics = c
        .create_course(&w.admin, science.id, course("PHY101"), Place::Last)
        .await
        .unwrap();
    customise
        .set_flag(&w.admin, "courses.groups", science.scope, false, false)
        .await
        .unwrap();
    assert_eq!(
        error(
            c.create_group(&w.admin, physics.id, group("Red", GroupVisibility::Everyone))
                .await
        )
        .1,
        "customise.feature_off"
    );
    customise
        .set_flag(&w.admin, "courses.groups", physics.scope, true, false)
        .await
        .unwrap();
    c.create_group(&w.admin, physics.id, group("Red", GroupVisibility::Everyone))
        .await
        .unwrap();
    w.drop().await;
}
