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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of access, against PostgreSQL, on a campus with nested
//! scopes, the default roles and a few people.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a failing setup must stop the test"
)]
#![allow(clippy::too_many_lines, reason = "each rule reads as one story")]

use std::collections::BTreeMap;
use std::sync::Arc;

use kampus_access::permissions::{self, GRANT_CREATE};
use kampus_access::{
    Access, Actor, PermissionDeclaration, PermissionKind, PrincipalId, Risk, RoleId, RoleTemplateDeclaration,
    ScopeId, ScopeKind, Value,
};
use kampus_common::{ErrorKind, FixedClock, Timestamp};
use kampus_db::{Database, Prefix, Upgrader};

/// Read a course and its content.
const VIEW: &str = "courses.course.view";
/// Change a course's settings; meaningful down to courses.
const UPDATE: &str = "courses.course.update";
/// Edit content; meaningful down to activities.
const EDIT: &str = "courses.content.edit";
/// Add content that runs in browsers.
const SCRIPT: &str = "courses.content.script";

/// The permissions a courses module would declare.
const COURSE_PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: VIEW,
        kind: PermissionKind::Read,
        risks: &[],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: UPDATE,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: EDIT,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: SCRIPT,
        kind: PermissionKind::Write,
        risks: &[Risk::ScriptInjection],
        lowest_kind: ScopeKind::Activity,
    },
];

/// The templates of the tests: the module's, with the courses permissions added.
const TEMPLATES: &[RoleTemplateDeclaration] = &[
    RoleTemplateDeclaration {
        code: "teacher",
        version: 1,
        values: &[
            (GRANT_CREATE, Value::Allow),
            (permissions::VIEW_AS, Value::Allow),
            (VIEW, Value::Allow),
            (UPDATE, Value::Allow),
            (EDIT, Value::Allow),
            (SCRIPT, Value::Allow),
        ],
    },
    RoleTemplateDeclaration {
        code: "learner",
        version: 1,
        values: &[(VIEW, Value::Allow)],
    },
    RoleTemplateDeclaration {
        code: "guest",
        version: 1,
        values: &[
            (VIEW, Value::Allow),
            (UPDATE, Value::Allow),
            (SCRIPT, Value::Allow),
        ],
    },
];

/// A campus with its scopes, roles and people.
struct World {
    /// The service.
    access: Access,
    /// The clock of the service.
    clock: Arc<FixedClock>,
    /// The database, to drop at the end.
    db: Database,
    /// The campus.
    campus: ScopeId,
    /// A category of the campus.
    category: ScopeId,
    /// A course of the category.
    course: ScopeId,
    /// Another course of the category.
    other_course: ScopeId,
    /// A section of the course.
    section: ScopeId,
    /// A section nested in the first.
    subsection: ScopeId,
    /// An activity of the nested section.
    activity: ScopeId,
    /// The default roles of the campus.
    roles: BTreeMap<&'static str, RoleId>,
    /// The campus administrator.
    admin: Actor,
    /// A manager of the campus.
    manager: Actor,
    /// A teacher of the course.
    teacher: Actor,
    /// A learner of the course.
    learner: Actor,
    /// Someone with no roles.
    outsider: Actor,
}

/// The principal of an actor.
fn id(actor: Actor) -> PrincipalId {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => p,
        Actor::Visitor(_) => unreachable!("visitors are not principals"),
    }
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("a{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    Upgrader::new(&db, &[kampus_access::MODULE.schema()], "0.1.0")
        .apply(true)
        .await
        .unwrap();
    let clock = Arc::new(FixedClock::new(
        Timestamp::parse_rfc3339("2026-11-18T10:00:00Z").unwrap(),
    ));
    let access = Access::new(db.clone(), clock.clone());
    let mut templates: Vec<RoleTemplateDeclaration> = permissions::TEMPLATES
        .iter()
        .filter(|t| !TEMPLATES.iter().any(|x| x.code == t.code))
        .copied()
        .collect();
    templates.extend_from_slice(TEMPLATES);
    access
        .sync(
            &[
                ("access", permissions::PERMISSIONS),
                ("courses", COURSE_PERMISSIONS),
            ],
            &templates,
        )
        .await
        .unwrap();

    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let category = access.create_scope(campus, ScopeKind::Category).await.unwrap();
    let course = access.create_scope(category, ScopeKind::Course).await.unwrap();
    let other_course = access.create_scope(category, ScopeKind::Course).await.unwrap();
    let section = access.create_scope(course, ScopeKind::Section).await.unwrap();
    let subsection = access.create_scope(section, ScopeKind::Section).await.unwrap();
    let activity = access
        .create_scope(subsection, ScopeKind::Activity)
        .await
        .unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();

    let person = || async {
        let id = PrincipalId::generate();
        access.create_principal(id, campus).await.unwrap();
        Actor::Principal(id)
    };
    let (admin, manager, teacher, learner, outsider) = (
        person().await,
        person().await,
        person().await,
        person().await,
        person().await,
    );
    access
        .grant_for_module("setup", roles["campus_administrator"], campus, id(admin), None)
        .await
        .unwrap();
    access
        .grant(&admin, roles["manager"], campus, id(manager), None)
        .await
        .unwrap();
    access
        .grant(&admin, roles["teacher"], course, id(teacher), None)
        .await
        .unwrap();
    access
        .grant(&teacher, roles["learner"], course, id(learner), None)
        .await
        .unwrap();
    World {
        access,
        clock,
        db,
        campus,
        category,
        course,
        other_course,
        section,
        subsection,
        activity,
        roles,
        admin,
        manager,
        teacher,
        learner,
        outsider,
    }
}

impl World {
    /// Whether an actor may, and the rule that decided.
    async fn can(&self, actor: &Actor, permission: &str, scope: ScopeId) -> (bool, &'static str) {
        let decision = self.access.check(actor, permission, scope).await.unwrap();
        (decision.allowed, decision.rule)
    }

    /// Drops the installation.
    async fn drop(self) {
        for module in ["db", "access"] {
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
async fn scopes_form_one_tree_of_allowed_kinds() {
    let w = world().await;
    let error = w
        .access
        .create_scope(w.campus, ScopeKind::Course)
        .await
        .unwrap_err();
    assert_eq!(
        error.code(),
        "access.scope_parent_invalid",
        "a course hangs from a category, not a campus"
    );
    assert!(
        w.access.create_scope(w.campus, ScopeKind::User).await.is_ok(),
        "user scopes hang from their campus"
    );
    assert!(
        w.access.create_scope(w.activity, ScopeKind::Widget).await.is_ok(),
        "a widget hangs from any scope that shows it"
    );
    assert_eq!(
        w.access.create_platform().await.unwrap_err().code(),
        "access.platform_exists",
        "there is one tree"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_section_is_a_scope() {
    let w = world().await;
    let leader = PrincipalId::generate();
    w.access.create_principal(leader, w.campus).await.unwrap();
    w.access
        .grant(&w.admin, w.roles["section_leader"], w.section, leader, None)
        .await
        .unwrap();
    w.access
        .set_role_value(&w.admin, w.roles["section_leader"], EDIT, Some(Value::Allow))
        .await
        .unwrap();
    let leader = Actor::Principal(leader);
    assert!(
        w.can(&leader, EDIT, w.activity).await.0,
        "a role granted on a section reaches its nested sections and activities"
    );
    assert!(
        !w.can(&leader, EDIT, w.course).await.0,
        "but not the course around it"
    );
    let sibling = w.access.create_scope(w.course, ScopeKind::Section).await.unwrap();
    assert!(!w.can(&leader, EDIT, sibling).await.0, "nor a sibling section");
    w.drop().await;
}

#[tokio::test]
async fn only_active_grants_count() {
    let w = world().await;
    let temporary = PrincipalId::generate();
    w.access.create_principal(temporary, w.campus).await.unwrap();
    let until = Timestamp::parse_rfc3339("2026-11-18T11:00:00Z").unwrap();
    w.access
        .grant(&w.admin, w.roles["teacher"], w.course, temporary, Some(until))
        .await
        .unwrap();
    assert!(
        w.can(&Actor::Principal(temporary), EDIT, w.activity).await.0,
        "an active grant on an ancestor counts"
    );
    w.clock
        .set(Timestamp::parse_rfc3339("2026-11-18T12:00:00Z").unwrap());
    assert!(
        !w.can(&Actor::Principal(temporary), EDIT, w.activity).await.0,
        "an expired grant does not"
    );
    w.drop().await;
}

#[tokio::test]
async fn the_nearest_override_up_to_the_campus_decides_for_each_role() {
    let w = world().await;
    let teacher = w.roles["teacher"];
    w.access
        .set_override(&w.admin, w.section, teacher, EDIT, Some(Value::Deny))
        .await
        .unwrap();
    assert!(
        !w.can(&w.teacher, EDIT, w.subsection).await.0,
        "a deny in the section applies below it"
    );
    assert!(w.can(&w.teacher, EDIT, w.course).await.0, "but not above it");
    w.access
        .set_override(&w.admin, w.activity, teacher, EDIT, Some(Value::Allow))
        .await
        .unwrap();
    assert!(
        w.can(&w.teacher, EDIT, w.activity).await.0,
        "a nearer allow wins over a deny further up"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_forbid_on_any_role_fails_the_check() {
    let w = world().await;
    w.access
        .grant(&w.teacher, w.roles["learner"], w.course, id(w.teacher), None)
        .await
        .unwrap();
    w.access
        .set_override(&w.admin, w.course, w.roles["learner"], EDIT, Some(Value::Forbid))
        .await
        .unwrap();
    assert_eq!(
        w.can(&w.teacher, EDIT, w.activity).await,
        (false, "forbidden"),
        "the learner role's forbid beats the teacher role's allow"
    );
    w.drop().await;
}

#[tokio::test]
async fn one_allow_passes_and_none_fails() {
    let w = world().await;
    assert_eq!(w.can(&w.teacher, EDIT, w.activity).await, (true, "allowed"));
    assert_eq!(
        w.can(&w.outsider, VIEW, w.course).await,
        (false, "no_allow"),
        "no role, no allow"
    );
    assert_eq!(
        w.can(&w.learner, EDIT, w.course).await,
        (false, "no_allow"),
        "a role without a value does not allow"
    );
    w.drop().await;
}

#[tokio::test]
async fn there_is_no_bypass() {
    let w = world().await;
    assert!(
        !w.can(&w.admin, VIEW, w.course).await.0,
        "an administrator holds only what their roles allow"
    );
    assert!(w.can(&w.admin, GRANT_CREATE, w.course).await.0);
    w.drop().await;
}

#[tokio::test]
async fn a_campus_is_never_left_unmanaged() {
    let w = world().await;
    let admin_grant = w
        .access
        .check(&w.admin, GRANT_CREATE, w.campus)
        .await
        .unwrap()
        .grants
        .iter()
        .find(|g| g.role == w.roles["campus_administrator"])
        .unwrap()
        .grant
        .unwrap();
    w.access.revoke_for_module("setup", admin_grant).await.unwrap();
    let manager_role = w.roles["manager"];
    let error = w
        .access
        .set_role_value(&w.manager, manager_role, GRANT_CREATE, None)
        .await
        .unwrap_err();
    assert_eq!(
        error.kind(),
        ErrorKind::PermissionDenied,
        "the manager may not manage roles at all"
    );
    let manager_grant = w
        .access
        .check(&w.manager, GRANT_CREATE, w.campus)
        .await
        .unwrap()
        .grants[0]
        .grant
        .unwrap();
    let error = w.access.revoke(&w.manager, manager_grant).await.unwrap_err();
    assert_eq!(
        (error.kind(), error.code()),
        (ErrorKind::Conflict, "access.campus_unmanaged"),
        "the last person able to grant cannot be removed"
    );
    w.drop().await;
}

#[tokio::test]
async fn visitors_and_guests_never_write_or_take_risks() {
    let w = world().await;
    let visitor = Actor::Visitor(w.campus);
    assert!(
        w.can(&visitor, VIEW, w.course).await.0,
        "visitors read what the guest role reads"
    );
    assert_eq!(
        w.can(&visitor, UPDATE, w.course).await,
        (false, "guest_limit"),
        "the guest role's allow of a write is ignored"
    );
    assert_eq!(
        w.can(&visitor, SCRIPT, w.activity).await,
        (false, "guest_limit"),
        "and of a risky permission"
    );
    let guest = PrincipalId::generate();
    w.access.create_principal(guest, w.campus).await.unwrap();
    w.access
        .grant_for_module("enrol", w.roles["guest"], w.course, guest, None)
        .await
        .unwrap();
    assert_eq!(
        w.can(&Actor::Principal(guest), UPDATE, w.course).await,
        (false, "guest_limit"),
        "a granted guest role is limited too"
    );
    w.drop().await;
}

#[tokio::test]
async fn inactive_accounts_fail_every_check() {
    let w = world().await;
    w.access.set_principal_active(id(w.teacher), false).await.unwrap();
    assert_eq!(
        w.can(&w.teacher, VIEW, w.course).await,
        (false, "inactive_account")
    );
    w.access.set_principal_active(id(w.teacher), true).await.unwrap();
    assert!(w.can(&w.teacher, VIEW, w.course).await.0);
    w.drop().await;
}

#[tokio::test]
async fn a_frozen_scope_only_reads() {
    let w = world().await;
    w.access.set_frozen(&w.admin, w.course, true).await.unwrap();
    assert_eq!(
        w.can(&w.teacher, EDIT, w.activity).await,
        (false, "frozen_scope"),
        "nothing is written in or below a frozen scope"
    );
    assert!(w.can(&w.teacher, VIEW, w.activity).await.0, "reading goes on");
    assert_eq!(
        w.access
            .set_frozen(&w.teacher, w.course, false)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied,
        "unfreezing needs the permission"
    );
    w.access.set_frozen(&w.admin, w.course, false).await.unwrap();
    assert!(w.can(&w.teacher, EDIT, w.activity).await.0);
    w.drop().await;
}

#[tokio::test]
async fn overrides_stop_at_the_lowest_kind_of_a_permission() {
    let w = world().await;
    let error = w
        .access
        .set_override(&w.admin, w.section, w.roles["teacher"], UPDATE, Some(Value::Deny))
        .await
        .unwrap_err();
    assert_eq!(
        error.code(),
        "access.override_below_lowest_kind",
        "a course setting has no meaning in a section"
    );
    assert!(
        w.access
            .set_override(&w.admin, w.course, w.roles["teacher"], UPDATE, Some(Value::Deny))
            .await
            .is_ok()
    );
    w.drop().await;
}

#[tokio::test]
async fn granting_needs_the_permission_an_allowance_and_an_assignable_kind() {
    let w = world().await;
    let someone = PrincipalId::generate();
    w.access.create_principal(someone, w.campus).await.unwrap();
    assert!(
        w.access
            .grant(&w.teacher, w.roles["learner"], w.course, someone, None)
            .await
            .is_ok()
    );
    let no_allowance = w
        .access
        .grant(&w.teacher, w.roles["manager"], w.course, someone, None)
        .await
        .unwrap_err();
    assert_eq!(
        no_allowance.kind(),
        ErrorKind::PermissionDenied,
        "a teacher may not make managers"
    );
    let no_permission = w
        .access
        .grant(&w.learner, w.roles["learner"], w.course, someone, None)
        .await
        .unwrap_err();
    assert_eq!(no_permission.kind(), ErrorKind::PermissionDenied);
    let wrong_kind = w
        .access
        .grant(&w.admin, w.roles["teacher"], w.category, someone, None)
        .await
        .unwrap_err();
    assert_eq!(
        wrong_kind.code(),
        "access.role_not_assignable",
        "teachers are granted in courses and sections"
    );
    let duplicate = w
        .access
        .grant(&w.teacher, w.roles["learner"], w.course, someone, None)
        .await
        .unwrap_err();
    assert_eq!(
        duplicate.code(),
        "db.unique_violation",
        "one manual grant per role, scope and person"
    );
    w.drop().await;
}

#[tokio::test]
async fn overriding_needs_the_permission_an_allowance_and_risky_rights() {
    let w = world().await;
    let denied = w
        .access
        .set_override(&w.teacher, w.course, w.roles["learner"], EDIT, Some(Value::Allow))
        .await
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        ErrorKind::PermissionDenied,
        "teachers do not override"
    );
    let risky = w
        .access
        .set_override(
            &w.manager,
            w.course,
            w.roles["learner"],
            SCRIPT,
            Some(Value::Allow),
        )
        .await
        .unwrap_err();
    assert_eq!(
        risky.kind(),
        ErrorKind::PermissionDenied,
        "a risky permission needs access.override.risky"
    );
    assert!(
        w.access
            .set_override(&w.manager, w.course, w.roles["learner"], EDIT, Some(Value::Allow))
            .await
            .is_ok()
    );
    assert!(
        w.access
            .set_override(&w.admin, w.course, w.roles["learner"], SCRIPT, Some(Value::Allow))
            .await
            .is_ok()
    );
    w.drop().await;
}

#[tokio::test]
async fn inherit_deletes_the_override_and_every_change_is_recorded() {
    let w = world().await;
    let learner = w.roles["learner"];
    w.access
        .set_override(&w.admin, w.course, learner, EDIT, Some(Value::Allow))
        .await
        .unwrap();
    assert!(w.can(&w.learner, EDIT, w.course).await.0);
    w.access
        .set_override(&w.admin, w.course, learner, EDIT, None)
        .await
        .unwrap();
    assert!(
        !w.can(&w.learner, EDIT, w.course).await.0,
        "inherit removes the override"
    );
    let changes: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {}.change where entity = 'permission_override' and actor = $1",
        w.db.prefix().schema("access")
    )))
    .bind(id(w.admin).as_uuid())
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(changes, 2, "both changes are recorded with their actor");
    w.drop().await;
}

#[tokio::test]
async fn a_module_owns_the_grants_it_makes() {
    let w = world().await;
    let someone = PrincipalId::generate();
    w.access.create_principal(someone, w.campus).await.unwrap();
    let grant = w
        .access
        .grant_for_module("enrol", w.roles["learner"], w.course, someone, None)
        .await
        .unwrap();
    assert_eq!(
        w.access.revoke(&w.admin, grant).await.unwrap_err().code(),
        "access.grant_owned_by_module"
    );
    assert_eq!(
        w.access
            .revoke_for_module("groups", grant)
            .await
            .unwrap_err()
            .code(),
        "access.grant_not_owned"
    );
    assert!(w.access.revoke_for_module("enrol", grant).await.is_ok());
    w.drop().await;
}

#[tokio::test]
async fn roles_come_from_templates_and_reset_only_when_confirmed() {
    let w = world().await;
    let unknown = w
        .access
        .create_role(
            &w.admin,
            w.campus,
            "tutor",
            "no-such-template",
            &[ScopeKind::Course],
        )
        .await
        .unwrap_err();
    assert_eq!(
        unknown.code(),
        "access.template_not_found",
        "an unknown template is an error, never an empty role"
    );
    let tutor = w
        .access
        .create_role(&w.admin, w.campus, "Tutor", "teacher", &[ScopeKind::Course])
        .await
        .unwrap();
    assert_eq!(w.access.template_differences(tutor).await.unwrap().len(), 0);
    w.access
        .set_role_value(&w.admin, tutor, EDIT, Some(Value::Deny))
        .await
        .unwrap();
    let preview = w.access.reset_to_template(&w.admin, tutor, false).await.unwrap();
    assert_eq!(preview.len(), 1);
    assert_eq!(
        (preview[0].role, preview[0].template),
        (Some(Value::Deny), Some(Value::Allow))
    );
    assert_eq!(
        w.access.template_differences(tutor).await.unwrap().len(),
        1,
        "a preview changes nothing"
    );
    w.access.reset_to_template(&w.admin, tutor, true).await.unwrap();
    assert_eq!(w.access.template_differences(tutor).await.unwrap().len(), 0);
    w.drop().await;
}

#[tokio::test]
async fn view_as_is_read_only_scoped_and_ends() {
    let w = world().await;
    let session = w
        .access
        .start_view_as(id(w.teacher), w.course, w.roles["learner"])
        .await
        .unwrap();
    let viewing = Actor::ViewAs(id(w.teacher), session);
    assert!(
        w.can(&viewing, VIEW, w.activity).await.0,
        "reads as the learner role"
    );
    assert_eq!(
        w.can(&viewing, EDIT, w.activity).await,
        (false, "view_as_limit"),
        "never writes"
    );
    assert_eq!(
        w.can(&viewing, VIEW, w.other_course).await,
        (false, "view_as_limit"),
        "stays in its scope"
    );
    let manager = w
        .access
        .start_view_as(id(w.teacher), w.course, w.roles["manager"])
        .await
        .unwrap_err();
    assert_eq!(
        manager.kind(),
        ErrorKind::PermissionDenied,
        "only roles the actor may view as"
    );
    w.access.end_view_as(id(w.teacher), session).await.unwrap();
    assert!(
        !w.can(&viewing, VIEW, w.activity).await.0,
        "an ended session reads nothing"
    );
    w.drop().await;
}

#[tokio::test]
async fn every_check_explains_itself() {
    let w = world().await;
    w.access
        .set_override(&w.admin, w.section, w.roles["teacher"], EDIT, Some(Value::Deny))
        .await
        .unwrap();
    let decision = w.access.check(&w.teacher, EDIT, w.activity).await.unwrap();
    assert!(!decision.allowed);
    assert_eq!(decision.grants.len(), 1);
    let evaluation = &decision.grants[0];
    assert_eq!(
        (
            evaluation.role,
            evaluation.scope,
            evaluation.value,
            evaluation.set_at
        ),
        (w.roles["teacher"], w.course, Some(Value::Deny), Some(w.section))
    );
    let json = serde_json::to_value(&decision).unwrap();
    assert_eq!(json["rule"], "no_allow");
    w.drop().await;
}

#[tokio::test]
async fn lists_filter_by_the_scopes_where_a_permission_is_allowed() {
    let w = world().await;
    w.access
        .set_override(
            &w.admin,
            w.subsection,
            w.roles["teacher"],
            VIEW,
            Some(Value::Deny),
        )
        .await
        .unwrap();
    let scopes = w.access.allowed_scopes(&w.teacher, VIEW).await.unwrap();
    assert!(scopes.contains(&w.course) && scopes.contains(&w.section));
    assert!(
        !scopes.contains(&w.subsection) && !scopes.contains(&w.activity),
        "a deny lower down removes its subtree"
    );
    assert!(
        !scopes.contains(&w.other_course),
        "nothing outside the actor's grants"
    );
    w.drop().await;
}

#[tokio::test]
async fn undeclared_and_retired_permissions_fail() {
    let w = world().await;
    assert_eq!(
        w.can(&w.teacher, "courses.course.teleport", w.course).await,
        (false, "unknown_permission")
    );
    w.access
        .sync(
            &[
                ("access", permissions::PERMISSIONS),
                ("courses", &COURSE_PERMISSIONS[..3]),
            ],
            &[],
        )
        .await
        .unwrap();
    assert_eq!(
        w.can(&w.teacher, SCRIPT, w.activity).await,
        (false, "unknown_permission"),
        "a permission no module declares any more is retired"
    );
    w.drop().await;
}
