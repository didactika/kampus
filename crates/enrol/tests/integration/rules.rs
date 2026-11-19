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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of enrolment, against PostgreSQL: manual, self and system group enrolments,
//! their grants, capacity, keys, approvals, participation and expiry.

use std::sync::Arc;

use kampus_access::{Access, Actor, PrincipalId, RoleId, ScopeKind};
use kampus_common::{ErrorKind, Result, SystemClock, Timestamp};
use kampus_courses::{
    CategoryFields, Course, CourseFields, CourseState, Courses, GroupFields, GroupVisibility, Place,
    Visibility,
};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_enrol::{
    Enrol, EnrolmentChange, ExpiryAction, InstanceFields, Joined, Method, Participation, Period, Status,
};
use kampus_identity::{
    FixedPolicies, Identity, MemoryCourier, Notice, PasswordCost, PasswordHasher, SecretKey,
};
use kampus_users::{GroupKind, Users};

/// The password of every account of these tests.
const PASSWORD: &str = "a long and quiet river";

/// A cheap cost for the hashes of these tests.
const COST: PasswordCost = PasswordCost {
    memory_kib: 64,
    iterations: 1,
    parallelism: 1,
};

/// A campus with a course, its administrator and its roles.
struct World {
    /// The service.
    enrol: Enrol,
    /// Courses.
    courses: Courses,
    /// Users.
    users: Users,
    /// Access.
    access: Access,
    /// Identity.
    identity: Identity,
    /// The messages identity sent.
    courier: Arc<MemoryCourier>,
    /// The database.
    db: Database,
    /// The campus.
    campus: kampus_access::ScopeId,
    /// The roles of the campus.
    roles: std::collections::BTreeMap<&'static str, RoleId>,
    /// The administrator.
    admin: Actor,
    /// A course of the campus.
    course: Course,
}

/// The error kind and code of a result.
fn error<T: std::fmt::Debug>(result: Result<T>) -> (ErrorKind, String) {
    let error = result.expect_err("the operation fails");
    (error.kind(), error.code().to_owned())
}

/// The principal of an actor.
fn id(actor: &Actor) -> PrincipalId {
    match actor {
        Actor::Principal(p) => *p,
        _ => unreachable!("a person"),
    }
}

/// The course of these tests, in its category.
async fn first_course(courses: &Courses, admin: &Actor, campus: kampus_access::ScopeId) -> Course {
    let category = courses
        .create_category(
            admin,
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
    courses
        .create_course(
            admin,
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
        .unwrap()
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("e{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 6).await.unwrap();
    let modules = [
        &kampus_events::MODULE,
        &kampus_access::MODULE,
        &kampus_identity::MODULE,
        &kampus_users::MODULE,
        &kampus_customise::MODULE,
        &kampus_courses::MODULE,
        &kampus_enrol::MODULE,
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
    .with_password_cost(COST);
    let admin = Actor::Principal(
        identity
            .install_administrator(campus, "admin@example.org", "admin", PASSWORD)
            .await
            .unwrap(),
    );
    let users = Users::new(db.clone(), access.clone(), identity.clone(), Vec::new());
    let courses = Courses::new(
        db.clone(),
        access.clone(),
        clock.clone(),
        Arc::new(Participation::new(db.clone())),
    );
    let enrol = Enrol::new(
        db.clone(),
        access.clone(),
        courses.clone(),
        identity.clone(),
        users.clone(),
        clock,
        PasswordHasher::new(COST),
    );
    let course = first_course(&courses, &admin, campus).await;
    World {
        enrol,
        courses,
        users,
        access,
        identity,
        courier,
        db,
        campus,
        roles,
        admin,
        course,
    }
}

impl World {
    /// Invites a person to the campus and accepts the invitation.
    async fn person(&self, email: &str) -> Actor {
        let id = self
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

    /// The fields of an instance giving a role.
    fn fields(&self, name: &str, role: &str) -> InstanceFields {
        InstanceFields {
            name: name.to_owned(),
            role: self.roles[role],
            enabled: true,
            open_from: None,
            open_until: None,
            default_days: None,
            capacity: None,
            requires_approval: false,
            expiry_action: ExpiryAction::Suspend,
        }
    }

    /// Whether an actor may edit the course, which the teacher role allows.
    async fn teaches(&self, actor: &Actor) -> bool {
        self.access
            .check(actor, "courses.course.update", self.course.scope)
            .await
            .unwrap()
            .allowed
    }

    /// Drops the installation.
    async fn drop(self) {
        for module in [
            "enrol",
            "courses",
            "customise",
            "users",
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
async fn a_manual_enrolment_gives_its_role_and_removing_it_takes_only_its_own() {
    let w = world().await;
    let manual = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::Manual,
            w.fields("Manual", "teacher"),
            None,
            None,
        )
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    assert!(!w.teaches(&ana).await);
    let report = w
        .enrol
        .enrol(&w.admin, manual.id, &[id(&ana)], Period::default())
        .await
        .unwrap();
    let enrolment = report[0].1.as_ref().copied().unwrap();
    assert!(
        w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    assert!(w.teaches(&ana).await, "the enrolment gives its role");
    assert_eq!(
        error(
            w.enrol
                .enrol(&w.admin, manual.id, &[id(&ana)], Period::default())
                .await
                .unwrap()
                .remove(0)
                .1
        )
        .1,
        "enrol.already_enrolled",
        "one enrolment per person and instance"
    );

    w.access
        .grant(&w.admin, w.roles["teacher"], w.course.scope, id(&ana), None)
        .await
        .unwrap();
    w.enrol.remove_enrolment(&w.admin, enrolment, None).await.unwrap();
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    assert!(w.teaches(&ana).await, "the grant made by hand stays");
    w.drop().await;
}

#[tokio::test]
async fn whoever_configures_an_instance_must_be_able_to_grant_its_role() {
    let w = world().await;
    let bea = w.person("bea@example.org").await;
    w.access
        .grant(&w.admin, w.roles["teacher"], w.course.scope, id(&bea), None)
        .await
        .unwrap();
    assert_eq!(
        error(
            w.enrol
                .create_instance(
                    &bea,
                    w.course.id,
                    Method::Manual,
                    w.fields("Staff", "manager"),
                    None,
                    None
                )
                .await
        )
        .1,
        "access.no_allowance"
    );
    w.enrol
        .create_instance(
            &bea,
            w.course.id,
            Method::Manual,
            w.fields("Learners", "learner"),
            None,
            None,
        )
        .await
        .unwrap();
    w.drop().await;
}

#[tokio::test]
async fn capacity_is_never_exceeded_by_concurrent_enrolments() {
    let w = world().await;
    let limited = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::Manual,
            InstanceFields {
                capacity: Some(2),
                ..w.fields("Limited", "learner")
            },
            None,
            None,
        )
        .await
        .unwrap();
    let mut people = Vec::new();
    for i in 0..5 {
        people.push(id(&w.person(&format!("p{i}@example.org")).await));
    }
    let mut tasks = Vec::new();
    for person in people {
        let enrol = w.enrol.clone();
        let admin = w.admin;
        let instance = limited.id;
        tasks.push(tokio::spawn(async move {
            enrol
                .enrol(&admin, instance, &[person], Period::default())
                .await
                .unwrap()
                .remove(0)
                .1
        }));
    }
    let mut enrolled = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(_) => enrolled += 1,
            Err(e) => assert_eq!(e.code(), "enrol.instance_full"),
        }
    }
    assert_eq!(enrolled, 2);
    w.drop().await;
}

#[tokio::test]
async fn a_self_instance_takes_its_key_puts_people_in_the_group_of_a_group_key_and_limits_attempts() {
    let w = world().await;
    let open = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::SelfEnrol,
            w.fields("Self", "learner"),
            Some("physics-2026"),
            None,
        )
        .await
        .unwrap();
    let red = w
        .courses
        .create_group(
            &w.admin,
            w.course.id,
            GroupFields {
                name: "Red".to_owned(),
                code: None,
                description: None,
                section: None,
                visibility: GroupVisibility::Everyone,
            },
        )
        .await
        .unwrap();
    w.enrol
        .set_key(&w.admin, open.id, Some(red.id), Some("red-team-2026"))
        .await
        .unwrap();

    let ana = w.person("ana@example.org").await;
    assert_eq!(
        error(w.enrol.self_enrol(&ana, open.id, Some("wrong"), None, None).await).1,
        "enrol.key_wrong"
    );
    let joined = w
        .enrol
        .self_enrol(&ana, open.id, Some("physics-2026"), None, None)
        .await
        .unwrap();
    assert!(matches!(joined, Joined::Enrolled { .. }));

    let bea = w.person("bea@example.org").await;
    w.enrol
        .self_enrol(&bea, open.id, Some("red-team-2026"), None, None)
        .await
        .unwrap();
    let members = w.courses.members(&w.admin, red.id).await.unwrap();
    assert_eq!(members.len(), 1, "the group key put her in the group");
    assert_eq!(members[0].account, id(&bea));

    let eve = w.person("eve@example.org").await;
    for _ in 0..5 {
        assert_eq!(
            error(w.enrol.self_enrol(&eve, open.id, Some("guess"), None, None).await).1,
            "enrol.key_wrong"
        );
    }
    assert_eq!(
        error(
            w.enrol
                .self_enrol(&eve, open.id, Some("physics-2026"), None, None)
                .await
        )
        .0,
        ErrorKind::RateLimited,
        "too many wrong keys wait, even with the right one"
    );
    w.drop().await;
}

#[tokio::test]
async fn an_instance_with_approval_waits_for_a_decision() {
    let w = world().await;
    let asked = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::SelfEnrol,
            InstanceFields {
                requires_approval: true,
                ..w.fields("On request", "learner")
            },
            None,
            None,
        )
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    let Joined::Requested { request } = w
        .enrol
        .self_enrol(&ana, asked.id, None, Some("I would like to join"), None)
        .await
        .unwrap()
    else {
        panic!("a request");
    };
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    assert_eq!(
        error(w.enrol.self_enrol(&ana, asked.id, None, None, None).await).1,
        "enrol.request_pending"
    );
    assert_eq!(
        error(w.enrol.decide_request(&ana, request, true, None).await).0,
        ErrorKind::PermissionDenied
    );
    let pending = w.enrol.pending_requests(&w.admin, w.course.id).await.unwrap();
    assert_eq!(pending.len(), 1);
    w.enrol
        .decide_request(&w.admin, request, true, None)
        .await
        .unwrap();
    assert!(
        w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    assert_eq!(
        error(w.enrol.decide_request(&w.admin, request, false, None).await).1,
        "enrol.request_decided"
    );
    w.drop().await;
}

#[tokio::test]
async fn suspending_or_disabling_stops_participation_without_deleting_anything() {
    let w = world().await;
    let manual = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::Manual,
            w.fields("Manual", "teacher"),
            None,
            None,
        )
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    let enrolment = w
        .enrol
        .enrol(&w.admin, manual.id, &[id(&ana)], Period::default())
        .await
        .unwrap()
        .remove(0)
        .1
        .unwrap();
    let suspended = w
        .enrol
        .update_enrolment(
            &w.admin,
            enrolment,
            EnrolmentChange {
                status: Status::Suspended,
                starts_at: None,
                ends_at: None,
                reason: Some("on leave".to_owned()),
            },
            Some(1),
        )
        .await
        .unwrap();
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    assert!(!w.teaches(&ana).await, "a suspended enrolment gives no role");
    w.enrol
        .update_enrolment(
            &w.admin,
            enrolment,
            EnrolmentChange {
                status: Status::Active,
                starts_at: None,
                ends_at: None,
                reason: None,
            },
            Some(suspended.version),
        )
        .await
        .unwrap();
    assert!(w.teaches(&ana).await);

    w.enrol
        .update_instance(
            &w.admin,
            manual.id,
            InstanceFields {
                enabled: false,
                ..w.fields("Manual", "teacher")
            },
            None,
        )
        .await
        .unwrap();
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    assert_eq!(
        w.enrol
            .participants(&w.admin, w.course.id, false)
            .await
            .unwrap()
            .len(),
        1
    );
    w.drop().await;
}

#[tokio::test]
async fn a_system_group_instance_follows_its_group() {
    let w = world().await;
    let cohort = w
        .users
        .create_group(&w.admin, w.campus, "C2026", "Cohort 2026", GroupKind::Manual)
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    let bea = w.person("bea@example.org").await;
    w.users.add_member(&w.admin, cohort, id(&ana)).await.unwrap();
    w.users.add_member(&w.admin, cohort, id(&bea)).await.unwrap();
    let linked = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::SystemGroup,
            w.fields("Cohort", "learner"),
            None,
            Some(cohort),
        )
        .await
        .unwrap();
    let synced = w.enrol.sync_instance(&w.admin, linked.id).await.unwrap();
    assert_eq!((synced.added, synced.removed), (2, 0));
    let enrolment = w.enrol.participants(&w.admin, w.course.id, true).await.unwrap()[0].clone();
    assert_eq!(
        error(w.enrol.remove_enrolment(&w.admin, enrolment.id, None).await).1,
        "enrol.enrolment_linked",
        "a linked enrolment leaves with the group, not by hand"
    );
    w.users.remove_member(&w.admin, cohort, id(&bea)).await.unwrap();
    let synced = w.enrol.sync_all().await.unwrap();
    assert_eq!((synced.added, synced.removed), (0, 1));
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&bea))
            .await
            .unwrap()
    );
    assert!(
        w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    w.drop().await;
}

#[tokio::test]
async fn an_ended_enrolment_follows_its_expiry_rule_once() {
    let w = world().await;
    let manual = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::Manual,
            w.fields("Manual", "teacher"),
            None,
            None,
        )
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    let past = Period {
        starts_at: Some(Timestamp::parse_rfc3339("2025-09-01T00:00:00Z").unwrap()),
        ends_at: Some(Timestamp::parse_rfc3339("2026-06-30T00:00:00Z").unwrap()),
    };
    w.enrol
        .enrol(&w.admin, manual.id, &[id(&ana)], past)
        .await
        .unwrap()
        .remove(0)
        .1
        .unwrap();
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap(),
        "an enrolment whose end passed gives no participation"
    );
    let expired = w.enrol.run_expiry().await.unwrap();
    assert_eq!(expired.suspended, 1);
    assert_eq!(
        w.enrol.run_expiry().await.unwrap().suspended,
        0,
        "it is handled once"
    );
    let enrolment = &w.enrol.participants(&w.admin, w.course.id, false).await.unwrap()[0];
    assert_eq!(enrolment.status, Status::Suspended);
    w.drop().await;
}

#[tokio::test]
async fn courses_hear_who_takes_part() {
    let w = world().await;
    let manual = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::Manual,
            w.fields("Manual", "learner"),
            None,
            None,
        )
        .await
        .unwrap();
    let red = w
        .courses
        .create_group(
            &w.admin,
            w.course.id,
            GroupFields {
                name: "Red".to_owned(),
                code: None,
                description: None,
                section: None,
                visibility: GroupVisibility::Everyone,
            },
        )
        .await
        .unwrap();
    let ana = w.person("ana@example.org").await;
    assert_eq!(
        error(w.courses.add_member(&w.admin, red.id, id(&ana)).await).1,
        "courses.member_not_participant",
        "only participants join groups"
    );
    w.enrol
        .enrol(&w.admin, manual.id, &[id(&ana)], Period::default())
        .await
        .unwrap();
    w.courses.add_member(&w.admin, red.id, id(&ana)).await.unwrap();
    w.courses
        .transition_course(&w.admin, w.course.id, CourseState::Published, None)
        .await
        .unwrap();
    assert_eq!(
        error(
            w.courses
                .transition_course(&w.admin, w.course.id, CourseState::Draft, None)
                .await
        )
        .1,
        "courses.transition_invalid",
        "a course with participants does not go back to draft"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_change_in_a_system_group_reaches_its_enrolments_through_its_event() {
    let w = world().await;
    let cohort = w
        .users
        .create_group(&w.admin, w.campus, "C2026", "Cohort 2026", GroupKind::Manual)
        .await
        .unwrap();
    let linked = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::SystemGroup,
            w.fields("Cohort", "learner"),
            None,
            Some(cohort),
        )
        .await
        .unwrap();
    assert_eq!(linked.source, Some(cohort));
    let services = kampus_module::ServiceContext::default().with(w.enrol.clone());
    let dispatcher = kampus_events::Dispatcher::new(
        kampus_events::Events::new(w.db.clone()),
        &[&kampus_users::MODULE, &kampus_enrol::MODULE],
        services,
    );
    let ana = w.person("ana@example.org").await;
    w.users.add_member(&w.admin, cohort, id(&ana)).await.unwrap();
    while dispatcher.run_once().await.unwrap() {}
    assert!(
        w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap(),
        "the listener enrolled the new member at once"
    );
    w.users.remove_member(&w.admin, cohort, id(&ana)).await.unwrap();
    while dispatcher.run_once().await.unwrap() {}
    assert!(
        !w.enrol
            .is_active_participant(w.course.id, id(&ana))
            .await
            .unwrap()
    );
    w.drop().await;
}

#[tokio::test]
async fn a_person_is_enrolled_and_found_by_the_identifier_a_system_gives_the_enrolment() {
    let w = world().await;
    let manual = w
        .enrol
        .create_instance(
            &w.admin,
            w.course.id,
            Method::Manual,
            w.fields("Manual", "learner"),
            None,
            None,
        )
        .await
        .unwrap();
    let sis = |reg: &str| kampus_db::references::ExternalReference::new("sis", reg).unwrap();
    let ana = w.person("ana@example.org").await;
    let bea = w.person("bea@example.org").await;
    let (enrolment, created) = w
        .enrol
        .upsert_enrolment_by_reference(&w.admin, manual.id, id(&ana), Period::default(), &sis("R-1"))
        .await
        .unwrap();
    assert!(created);
    let (again, created) = w
        .enrol
        .upsert_enrolment_by_reference(&w.admin, manual.id, id(&ana), Period::default(), &sis("R-1"))
        .await
        .unwrap();
    assert_eq!((again.id, created), (enrolment.id, false));
    assert_eq!(
        error(
            w.enrol
                .upsert_enrolment_by_reference(&w.admin, manual.id, id(&bea), Period::default(), &sis("R-1"))
                .await
        )
        .1,
        "enrol.reference_other_account"
    );
    let found = w
        .enrol
        .enrolment_by_reference(w.campus, &sis("R-1"))
        .await
        .unwrap();
    assert_eq!(found, Some(enrolment.id));
    let moodle = kampus_db::references::ExternalReference::new("moodle", "9").unwrap();
    let both = w
        .enrol
        .set_enrolment_reference(&w.admin, enrolment.id, &moodle)
        .await
        .unwrap();
    assert_eq!(both.len(), 2);
    w.enrol
        .remove_enrolment_reference(&w.admin, enrolment.id, "moodle")
        .await
        .unwrap();
    assert_eq!(
        w.enrol
            .enrolment_external_references(&w.admin, enrolment.id)
            .await
            .unwrap(),
        [sis("R-1")]
    );
    w.drop().await;
}
