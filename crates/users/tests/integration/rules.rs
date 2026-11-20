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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of users, against PostgreSQL: profiles, preferences and system groups.

use std::sync::Arc;

use kampus_access::{Access, Actor, PrincipalId, ScopeId, ScopeKind};
use kampus_common::{ErrorKind, Result, SystemClock};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_identity::{AccountId, FixedPolicies, Identity, MemoryCourier, Notice, PasswordCost, SecretKey};
use kampus_module::{PreferenceDeclaration, ValueKind};
use kampus_users::{GroupKind, GroupRule, Member, Origin, ProfileFields, RuleField, Users};
use serde_json::json;

/// The preferences the tests declare, as a notification module would.
const PREFERENCES: &[PreferenceDeclaration] = &[
    PreferenceDeclaration {
        key: "notify.digest",
        summary: "How often the digest arrives",
        kind: ValueKind::Choice {
            options: &["daily", "weekly"],
            default: "weekly",
        },
    },
    PreferenceDeclaration {
        key: "notify.enabled",
        summary: "Whether notifications arrive",
        kind: ValueKind::Boolean { default: true },
    },
];

/// A campus with a category, its administrator and people.
struct World {
    /// The service.
    users: Users,
    /// Access.
    access: Access,
    /// The database.
    db: Database,
    /// The campus.
    campus: ScopeId,
    /// A category of the campus.
    category: ScopeId,
    /// The campus administrator.
    admin: Actor,
    /// Identity, to create people.
    identity: Identity,
    /// The messages identity sent.
    courier: Arc<MemoryCourier>,
}

/// The error kind and code of a result.
fn error<T: std::fmt::Debug>(result: Result<T>) -> (ErrorKind, String) {
    let error = result.expect_err("the operation fails");
    (error.kind(), error.code().to_owned())
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("u{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let modules = [
        &kampus_events::MODULE,
        &kampus_access::MODULE,
        &kampus_identity::MODULE,
        &kampus_files::MODULE,
        &kampus_users::MODULE,
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
    let category = access.create_scope(campus, ScopeKind::Category).await.unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();
    let admin = PrincipalId::generate();
    access.create_principal(admin, campus).await.unwrap();
    access
        .grant_for_module("setup", roles["campus_administrator"], campus, admin, None)
        .await
        .unwrap();
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
    let users = Users::new(db.clone(), access.clone(), identity.clone(), PREFERENCES.to_vec());
    World {
        users,
        access,
        db,
        campus,
        category,
        admin: Actor::Principal(admin),
        identity,
        courier,
    }
}

impl World {
    /// Invites a person to the campus and accepts the invitation.
    async fn person(&self, email: &str) -> AccountId {
        let id = self
            .identity
            .invite(&self.admin, self.campus, email, None)
            .await
            .unwrap();
        let Some(Notice::Invitation { token }) = self.courier.last_to(email).map(|m| m.notice) else {
            panic!("an invitation was sent");
        };
        self.identity
            .accept_invitation(&token, "a long and quiet river")
            .await
            .unwrap();
        id
    }

    /// Drops the installation.
    async fn drop(self) {
        for module in ["users", "files", "identity", "access", "events", "db"] {
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

/// A profile with these names and language.
fn fields(given: &str, language: &str) -> ProfileFields {
    ProfileFields {
        given_name: Some(given.to_owned()),
        language_code: Some(language.to_owned()),
        time_zone: Some("Europe/Madrid".to_owned()),
        ..ProfileFields::default()
    }
}

#[tokio::test]
async fn everyone_edits_their_own_profile_with_if_match() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let me = Actor::Principal(ana);
    let empty = w.users.profile(&me, ana).await.unwrap();
    assert_eq!((empty.given_name, empty.version), (None, 0));

    let saved = w
        .users
        .update_profile(&me, ana, fields("Ana", "es"), Some(0))
        .await
        .unwrap();
    assert_eq!((saved.given_name.as_deref(), saved.version), (Some("Ana"), 1));
    let again = w
        .users
        .update_profile(&me, ana, fields("Ana María", "es"), Some(1))
        .await
        .unwrap();
    assert_eq!(again.version, 2);
    assert_eq!(
        error(
            w.users
                .update_profile(&me, ana, fields("Stale", "es"), Some(1))
                .await
        )
        .0,
        ErrorKind::PreconditionFailed,
        "a profile changed since it was read is not overwritten"
    );
    assert_eq!(
        w.users.profile(&me, ana).await.unwrap().given_name.as_deref(),
        Some("Ana María")
    );
    assert_eq!(
        error(
            w.users
                .update_profile(&me, ana, fields("Ana", "Spanish"), None)
                .await
        )
        .1,
        "users.profile_invalid"
    );
    w.drop().await;
}

#[tokio::test]
async fn someone_elses_profile_needs_its_permission() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let bea = w.person("bea@example.org").await;
    let as_bea = Actor::Principal(bea);
    assert_eq!(
        error(w.users.profile(&as_bea, ana).await).0,
        ErrorKind::PermissionDenied
    );
    assert_eq!(
        error(
            w.users
                .update_profile(&as_bea, ana, fields("Ana", "es"), None)
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
    w.users
        .update_profile(&w.admin, ana, fields("Ana", "es"), None)
        .await
        .unwrap();
    assert_eq!(
        w.users
            .profile(&w.admin, ana)
            .await
            .unwrap()
            .given_name
            .as_deref(),
        Some("Ana")
    );
    w.drop().await;
}

#[tokio::test]
async fn preferences_are_declared_typed_and_default() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let me = Actor::Principal(ana);
    assert_eq!(
        w.users.preferences(&me, ana).await.unwrap(),
        [
            ("notify.digest", json!("weekly")),
            ("notify.enabled", json!(true))
        ]
    );
    w.users
        .set_preference(&me, ana, "notify.digest", Some(json!("daily")))
        .await
        .unwrap();
    assert_eq!(w.users.preferences(&me, ana).await.unwrap()[0].1, json!("daily"));
    assert_eq!(
        error(
            w.users
                .set_preference(&me, ana, "notify.digest", Some(json!("hourly")))
                .await
        )
        .1,
        "users.preference_invalid"
    );
    assert_eq!(
        error(
            w.users
                .set_preference(&me, ana, "notify.enabled", Some(json!("yes")))
                .await
        )
        .1,
        "users.preference_invalid"
    );
    assert_eq!(
        error(
            w.users
                .set_preference(&me, ana, "theme.colour", Some(json!("red")))
                .await
        )
        .1,
        "users.preference_unknown"
    );
    w.users
        .set_preference(&me, ana, "notify.digest", None)
        .await
        .unwrap();
    assert_eq!(w.users.preferences(&me, ana).await.unwrap()[0].1, json!("weekly"));
    w.drop().await;
}

#[tokio::test]
async fn a_rule_or_a_sync_removes_only_the_memberships_it_made() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let bea = w.person("bea@example.org").await;
    let eva = w.person("eva@example.org").await;
    for (person, language) in [(ana, "es"), (bea, "es"), (eva, "en")] {
        w.users
            .update_profile(
                &Actor::Principal(person),
                person,
                fields("Someone", language),
                None,
            )
            .await
            .unwrap();
    }
    let rule = GroupKind::Rule(GroupRule {
        field: RuleField::LanguageCode,
        equals: "es".to_owned(),
    });
    assert_eq!(
        error(
            w.users
                .create_group(&Actor::Principal(ana), w.campus, "es", "Spanish", rule.clone())
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
    let group = w
        .users
        .create_group(&w.admin, w.category, "es", "Spanish speakers", rule)
        .await
        .unwrap();
    assert_eq!(
        error(
            w.users
                .create_group(&w.admin, w.category, "es", "Again", GroupKind::Manual)
                .await
        )
        .1,
        "users.group_exists"
    );

    assert_eq!(w.users.apply_rule(&w.admin, group).await.unwrap(), 2);
    w.users.add_member(&w.admin, group, ana).await.unwrap();
    w.users
        .replace_members(group, Origin::Sync, &[eva])
        .await
        .unwrap();
    let members = w.users.members(&w.admin, group).await.unwrap();
    assert_eq!(
        members.len(),
        4,
        "ana by rule and by hand, bea by rule, eva by sync"
    );

    w.users
        .update_profile(&Actor::Principal(ana), ana, fields("Someone", "en"), None)
        .await
        .unwrap();
    assert_eq!(w.users.apply_rule(&w.admin, group).await.unwrap(), 1);
    w.users.replace_members(group, Origin::Sync, &[]).await.unwrap();
    let members = w.users.members(&w.admin, group).await.unwrap();
    let mut expected = vec![
        Member {
            account: ana,
            origin: Origin::Manual,
        },
        Member {
            account: bea,
            origin: Origin::Rule,
        },
    ];
    expected.sort_by_key(|m| m.account.as_uuid());
    assert_eq!(
        members, expected,
        "the rule left ana's own membership, the sync left the others"
    );

    w.users.remove_member(&w.admin, group, ana).await.unwrap();
    assert_eq!(
        w.users.members(&w.admin, group).await.unwrap(),
        [Member {
            account: bea,
            origin: Origin::Rule,
        }],
        "removing by hand leaves the rule's memberships"
    );
    assert_eq!(
        error(w.users.replace_members(group, Origin::Manual, &[]).await).1,
        "users.origin_manual"
    );
    w.drop().await;
}

#[tokio::test]
async fn system_groups_live_in_a_campus_or_a_category() {
    let w = world().await;
    let course = w
        .access
        .create_scope(w.category, ScopeKind::Course)
        .await
        .unwrap();
    assert_eq!(
        error(
            w.users
                .create_group(&w.admin, course, "c", "Course", GroupKind::Manual)
                .await
        )
        .1,
        "users.group_invalid"
    );
    w.users
        .create_group(&w.admin, w.campus, "all", "Everyone", GroupKind::Manual)
        .await
        .unwrap();
    w.drop().await;
}

#[tokio::test]
async fn a_system_group_is_created_once_and_found_by_the_identifier_a_system_gives_it() {
    let w = world().await;
    let sis = |code: &str| kampus_db::references::ExternalReference::new("sis", code).unwrap();
    let (cohort, created) = w
        .users
        .ensure_group_by_reference(
            &w.admin,
            w.category,
            &sis("COH-2026"),
            ("c2026", "Cohort 2026"),
            GroupKind::Manual,
        )
        .await
        .unwrap();
    assert!(created);
    let (same, created) = w
        .users
        .ensure_group_by_reference(
            &w.admin,
            w.category,
            &sis("COH-2026"),
            ("other", "Other"),
            GroupKind::Manual,
        )
        .await
        .unwrap();
    assert_eq!((same, created), (cohort, false));
    assert_eq!(
        w.users
            .group_by_reference(w.campus, &sis("COH-2026"))
            .await
            .unwrap(),
        Some(cohort)
    );
    let manual = w
        .users
        .create_group(&w.admin, w.campus, "staff", "Staff", GroupKind::Manual)
        .await
        .unwrap();
    let taken = w
        .users
        .set_group_reference(&w.admin, manual, &sis("COH-2026"))
        .await
        .unwrap_err();
    assert_eq!(taken.code(), "references.taken");
    w.users
        .set_group_reference(&w.admin, manual, &sis("STAFF"))
        .await
        .unwrap();
    assert_eq!(
        w.users.group_external_references(&w.admin, manual).await.unwrap(),
        [sis("STAFF")]
    );
    w.users
        .remove_group_reference(&w.admin, manual, "sis")
        .await
        .unwrap();
    assert_eq!(
        w.users
            .remove_group_reference(&w.admin, manual, "sis")
            .await
            .unwrap_err()
            .code(),
        "references.not_found"
    );
    w.drop().await;
}
