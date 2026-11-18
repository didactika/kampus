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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of identity, against PostgreSQL, on a campus with its default roles, an administrator and a manager.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a failing setup must stop the test"
)]
#![allow(clippy::too_many_lines, reason = "each rule reads as one story")]

use std::collections::BTreeMap;
use std::sync::Arc;

use kampus_access::{Access, Actor, PrincipalId, RoleId, ScopeId, ScopeKind};
use kampus_common::{ErrorKind, FixedClock, Result, Timestamp};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_identity::permissions::{ACCOUNT_READ, TOKEN_MANAGE_OWN};
use kampus_identity::service::account::SignUp;
use kampus_identity::service::api_token::{TokenRequest, TokenScope};
use kampus_identity::service::oidc::{OidcOutcome, ProviderConfig};
use kampus_identity::service::second_factor::Proof;
use kampus_identity::service::session::Client;
use kampus_identity::service::sign_in::{Credential, SignIn, SignInOutcome};
use kampus_identity::support::totp;
use kampus_identity::{
    AccountId, AccountKind, AccountStatus, CampusPolicy, FactorKind, FixedPolicies, Identity, LinkingPolicy,
    MemoryCourier, MfaLevel, Notice, PasswordCost, ProviderId, SecretKey, Session,
};
use openidconnect::reqwest;
use openidconnect::url::Url;
use sqlx::Row;

/// A password that passes the default policy.
const PASSWORD: &str = "a long and quiet river";

/// A campus with its roles and people.
struct World {
    /// The service.
    identity: Identity,
    /// Access, on the same database.
    access: Access,
    /// The clock of both services.
    clock: Arc<FixedClock>,
    /// The messages identity sent.
    courier: Arc<MemoryCourier>,
    /// The database.
    db: Database,
    /// The campus.
    campus: ScopeId,
    /// Another campus.
    other_campus: ScopeId,
    /// The default roles of the campus.
    roles: BTreeMap<&'static str, RoleId>,
    /// The campus administrator, a principal set up by hand.
    admin: Actor,
    /// The administrator of the other campus.
    other_admin: Actor,
    /// A manager with an account.
    manager: AccountId,
}

/// The policy of the tests: self sign-up open, otherwise the defaults.
fn policy() -> CampusPolicy {
    CampusPolicy {
        self_sign_up: true,
        ..CampusPolicy::default()
    }
}

/// The error kind and code of a result.
fn error<T: std::fmt::Debug>(result: Result<T>) -> (ErrorKind, String) {
    let error = result.expect_err("the operation fails");
    (error.kind(), error.code().to_owned())
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    world_with(policy()).await
}

/// Builds the world with a policy.
async fn world_with(policy: CampusPolicy) -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("i{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    Upgrader::new(
        &db,
        &[kampus_access::MODULE.schema(), kampus_identity::MODULE.schema()],
        "0.1.0",
    )
    .apply(true)
    .await
    .unwrap();
    let clock = Arc::new(FixedClock::new(
        Timestamp::parse_rfc3339("2026-11-18T10:00:00Z").unwrap(),
    ));
    let access = Access::new(db.clone(), clock.clone());
    access
        .sync_modules(&[&kampus_access::MODULE, &kampus_identity::MODULE])
        .await
        .unwrap();
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let other_campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();
    let other_roles = access.install_campus_roles(other_campus).await.unwrap();
    let administrator = |campus: ScopeId, role: RoleId| {
        let access = access.clone();
        async move {
            let admin = PrincipalId::generate();
            access.create_principal(admin, campus).await.unwrap();
            access
                .grant_for_module("setup", role, campus, admin, None)
                .await
                .unwrap();
            Actor::Principal(admin)
        }
    };
    let admin = administrator(campus, roles["campus_administrator"]).await;
    let other_admin = administrator(other_campus, other_roles["campus_administrator"]).await;
    let courier = Arc::new(MemoryCourier::default());
    let identity = Identity::new(
        db.clone(),
        access.clone(),
        clock.clone(),
        Arc::new(FixedPolicies(policy)),
        courier.clone(),
        SecretKey::generate().unwrap(),
    )
    .with_password_cost(PasswordCost {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    });
    let mut world = World {
        identity,
        access,
        clock,
        courier,
        db,
        campus,
        other_campus,
        roles,
        admin,
        other_admin,
        manager: AccountId::generate(),
    };
    world.manager = world.person("manager@example.org").await;
    world
        .access
        .grant(&world.admin, world.roles["manager"], campus, world.manager, None)
        .await
        .unwrap();
    world
}

impl World {
    /// Invites a person to the campus and accepts the invitation with [`PASSWORD`].
    async fn person(&self, email: &str) -> AccountId {
        let id = self
            .identity
            .invite(&self.admin, self.campus, email, None)
            .await
            .unwrap();
        self.identity
            .accept_invitation(&self.token_sent_to(email), PASSWORD)
            .await
            .unwrap();
        id
    }

    /// The token of the last link sent to an email.
    fn token_sent_to(&self, email: &str) -> String {
        match self.courier.last_to(email).map(|m| m.notice) {
            Some(Notice::ConfirmEmail { token } | Notice::Invitation { token }) => token,
            other => panic!("no link was sent to {email}: {other:?}"),
        }
    }

    /// Signs in with a password, expecting a session.
    async fn session(&self, name: &str) -> (Session, String) {
        match self.sign_in(name, PASSWORD, None).await.unwrap() {
            SignInOutcome::SignedIn(new) => (new.session, new.token),
            SignInOutcome::SecondFactorRequired { .. } => panic!("no second factor was expected"),
        }
    }

    /// Signs in with a password from an address.
    async fn sign_in(&self, name: &str, password: &str, address: Option<&str>) -> Result<SignInOutcome> {
        self.identity
            .sign_in(
                self.campus,
                SignIn {
                    name: name.to_owned(),
                    credential: Credential::Password(password.to_owned()),
                    client: Client {
                        name: Some("tests".to_owned()),
                        address: address.map(str::to_owned),
                    },
                },
            )
            .await
    }

    /// Moves the clock forward.
    fn advance(&self, by: time::Duration) {
        let now = self.identity_now();
        self.clock.set(Timestamp::from_offset_date_time(now + by));
    }

    /// The instant of the clock.
    fn identity_now(&self) -> time::OffsetDateTime {
        use kampus_common::Clock;
        self.clock.now().as_offset_date_time()
    }

    /// The account as an administrator reads it.
    async fn account(&self, id: AccountId) -> kampus_identity::Account {
        self.identity.account(&self.admin, id).await.unwrap()
    }

    /// The quoted name of a table of identity.
    fn table(&self, name: &str) -> String {
        format!("{}.\"{name}\"", self.db.prefix().schema("identity"))
    }

    /// Counts the rows of a table of identity for an account.
    async fn count(&self, table: &str, account: AccountId) -> i64 {
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "select count(*) from {} where account_id = $1",
            self.table(table)
        )))
        .bind(account.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .unwrap()
    }
}

#[tokio::test]
async fn usernames_and_emails_are_unique_in_a_campus() {
    let w = world().await;
    w.identity
        .invite(&w.admin, w.campus, "Ana@Example.org", Some("Ana"))
        .await
        .unwrap();
    assert_eq!(
        error(
            w.identity
                .invite(&w.admin, w.campus, "ana@example.ORG", None)
                .await
        ),
        (ErrorKind::Conflict, "identity.account_exists".to_owned())
    );
    assert_eq!(
        error(
            w.identity
                .invite(&w.admin, w.campus, "other@example.org", Some("ANA"))
                .await
        ),
        (ErrorKind::Conflict, "identity.account_exists".to_owned())
    );
    let id = w
        .identity
        .invite(&w.other_admin, w.other_campus, "ana@example.org", Some("ana"))
        .await
        .unwrap();
    let account = w.identity.account(&w.other_admin, id).await.unwrap();
    assert_eq!(account.email.as_deref(), Some("ana@example.org"));
    assert_eq!(account.username.as_deref(), Some("ana"));
}

#[tokio::test]
async fn the_lifecycle_is_a_recorded_state_machine() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let account = w.account(ana).await;
    assert_eq!(account.status, AccountStatus::Active);
    assert!(account.email_verified_at.is_some());
    let ana_actor = Actor::Principal(ana);
    assert!(
        w.access
            .check(&ana_actor, TOKEN_MANAGE_OWN, w.campus)
            .await
            .unwrap()
            .allowed
    );

    assert_eq!(
        error(w.identity.suspend(&ana_actor, w.manager, "no reason", None).await).0,
        ErrorKind::PermissionDenied
    );
    let manager = Actor::Principal(w.manager);
    assert_eq!(
        error(
            w.identity
                .suspend(&manager, ana, "late", Some(account.version - 1))
                .await
        )
        .0,
        ErrorKind::PreconditionFailed
    );
    w.identity
        .suspend(&manager, ana, "unpaid fees", Some(account.version))
        .await
        .unwrap();
    let suspended = w.account(ana).await;
    assert_eq!(suspended.status, AccountStatus::Suspended);
    assert_eq!(suspended.status_reason.as_deref(), Some("unpaid fees"));
    assert!(
        !w.access
            .check(&ana_actor, TOKEN_MANAGE_OWN, w.campus)
            .await
            .unwrap()
            .allowed
    );
    assert_eq!(
        error(w.identity.suspend(&manager, ana, "again", None).await),
        (ErrorKind::Conflict, "identity.transition_not_allowed".to_owned())
    );
    assert_eq!(
        error(w.identity.suspend(&manager, w.manager, "myself", None).await),
        (ErrorKind::Conflict, "identity.own_account".to_owned())
    );
    w.identity.reactivate(&manager, ana, "paid", None).await.unwrap();
    assert!(
        w.access
            .check(&ana_actor, TOKEN_MANAGE_OWN, w.campus)
            .await
            .unwrap()
            .allowed
    );

    let history: Vec<(Option<String>, String, Option<String>)> = sqlx::query(sqlx::AssertSqlSafe(format!(
        "select from_status, to_status, reason from {} where account_id = $1 order by at, id",
        w.table("account_transition")
    )))
    .bind(ana.as_uuid())
    .fetch_all(w.db.pool())
    .await
    .unwrap()
    .iter()
    .map(|r| (r.get(0), r.get(1), r.get(2)))
    .collect();
    let steps: Vec<(Option<&str>, &str)> = history
        .iter()
        .map(|(from, to, _)| (from.as_deref(), to.as_str()))
        .collect();
    assert_eq!(
        steps,
        vec![
            (None, "invited"),
            (Some("invited"), "active"),
            (Some("active"), "suspended"),
            (Some("suspended"), "active"),
        ]
    );
    assert_eq!(history[2].2.as_deref(), Some("unpaid fees"));
}

#[tokio::test]
async fn every_sign_in_runs_the_same_pipeline() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let (session, _) = w.session("ANA@example.org ").await;
    assert_eq!(session.account, ana);
    assert!(w.account(ana).await.last_sign_in_at.is_some());
    let failed = (ErrorKind::Unauthenticated, "identity.sign_in_failed".to_owned());
    assert_eq!(
        error(w.sign_in("ana@example.org", "a wrong password", None).await),
        failed
    );

    w.identity
        .suspend(&Actor::Principal(w.manager), ana, "paused", None)
        .await
        .unwrap();
    assert_eq!(error(w.sign_in("ana@example.org", PASSWORD, None).await), failed);

    let service = w
        .identity
        .create_service_account(&w.admin, w.campus, "grades-sync")
        .await
        .unwrap();
    assert_eq!(w.account(service).await.kind, AccountKind::Service);
    assert_eq!(error(w.sign_in("grades-sync", PASSWORD, None).await), failed);

    let strict = world_with(CampusPolicy {
        second_factor_required: true,
        ..policy()
    })
    .await;
    strict.person("bea@example.org").await;
    match strict.sign_in("bea@example.org", PASSWORD, None).await.unwrap() {
        SignInOutcome::SecondFactorRequired { challenge, .. } => assert_eq!(challenge.len(), 43),
        SignInOutcome::SignedIn(_) => panic!("the campus requires a second factor"),
    }
}

#[tokio::test]
async fn passwords_follow_the_policy_and_are_hashed() {
    let w = world().await;
    w.identity
        .invite(&w.admin, w.campus, "carla.ruiz@example.org", Some("carla"))
        .await
        .unwrap();
    let token = w.token_sent_to("carla.ruiz@example.org");
    let rejected = w.identity.accept_invitation(&token, "short").await.unwrap_err();
    assert_eq!(rejected.code(), "identity.password_rejected");
    assert!(rejected.fields().iter().any(|f| f.rule == "min_length"));
    let rejected = w
        .identity
        .accept_invitation(&token, "my name is carla.ruiz indeed")
        .await
        .unwrap_err();
    assert!(rejected.fields().iter().any(|f| f.rule == "contains_name"));
    let carla = w.identity.accept_invitation(&token, PASSWORD).await.unwrap();

    let (hash, params): (String, String) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select hash, algorithm_params from {} where account_id = $1",
        w.table("password")
    )))
    .bind(carla.as_uuid())
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert!(hash.starts_with("$argon2id$v=19$m=64,t=1,p=1$"));
    assert!(!hash.contains(PASSWORD));
    assert_eq!(params, "m=64,t=1,p=1");
    assert!(
        w.courier
            .sent()
            .iter()
            .all(|m| !format!("{:?}", m.notice).contains(PASSWORD)),
        "a password is never sent"
    );
}

#[tokio::test]
async fn one_time_tokens_are_hashed_expire_and_work_once() {
    let w = world().await;
    w.identity
        .sign_up(
            w.campus,
            SignUp {
                email: "dani@example.org".to_owned(),
                accepted_terms: true,
            },
        )
        .await
        .unwrap();
    let first = w.token_sent_to("dani@example.org");
    let stored: Vec<Vec<u8>> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select hash from {}",
        w.table("one_time_token")
    )))
    .fetch_all(w.db.pool())
    .await
    .unwrap();
    assert!(
        stored
            .iter()
            .all(|h| h.len() == 32 && h.as_slice() != first.as_bytes())
    );

    w.advance(time::Duration::hours(25));
    let invalid = (ErrorKind::Validation, "identity.token_invalid".to_owned());
    assert_eq!(error(w.identity.confirm(&first, PASSWORD).await), invalid);

    w.identity
        .sign_up(
            w.campus,
            SignUp {
                email: "dani@example.org".to_owned(),
                accepted_terms: true,
            },
        )
        .await
        .unwrap();
    let second = w.token_sent_to("dani@example.org");
    assert_ne!(first, second);
    let dani = w.identity.confirm(&second, PASSWORD).await.unwrap();
    assert_eq!(w.account(dani).await.status, AccountStatus::Active);
    assert_eq!(error(w.identity.confirm(&second, PASSWORD).await), invalid);
}

#[tokio::test]
async fn answers_never_reveal_accounts_and_sign_in_is_throttled() {
    let w = world().await;
    let request = || SignUp {
        email: "eva@example.org".to_owned(),
        accepted_terms: true,
    };
    w.identity.sign_up(w.campus, request()).await.unwrap();
    let eva = w
        .identity
        .confirm(&w.token_sent_to("eva@example.org"), PASSWORD)
        .await
        .unwrap();
    w.identity.sign_up(w.campus, request()).await.unwrap();
    assert_eq!(
        w.courier.last_to("eva@example.org").map(|m| m.notice),
        Some(Notice::AlreadyRegistered)
    );
    assert_eq!(
        error(w.sign_in("nobody@example.org", PASSWORD, None).await),
        error(w.sign_in("eva@example.org", "a wrong password", None).await)
    );

    let throttle = CampusPolicy::default().throttle;
    for name in ["eva@example.org", "nobody@example.org"] {
        for _ in 1..throttle.slow_after {
            assert_eq!(
                error(w.sign_in(name, "a wrong password", None).await).0,
                ErrorKind::Unauthenticated
            );
        }
        let throttled = w.sign_in(name, PASSWORD, None).await.unwrap_err();
        assert_eq!(throttled.kind(), ErrorKind::RateLimited, "{name}");
        assert!(throttled.params().iter().any(|(k, _)| k == "retry_after"));
    }
    let mut failures = throttle.slow_after;
    while failures < throttle.lock_after {
        match w.sign_in("eva@example.org", "a wrong password", None).await {
            Err(e) if e.kind() == ErrorKind::RateLimited => {
                let (_, seconds) = e.params().iter().find(|(k, _)| k == "retry_after").unwrap();
                w.advance(time::Duration::seconds(seconds.parse().unwrap()));
            }
            Err(_) => failures += 1,
            Ok(_) => panic!("a wrong password never signs in"),
        }
    }
    w.advance(time::Duration::minutes(5));
    assert_eq!(
        error(w.sign_in("eva@example.org", PASSWORD, None).await).0,
        ErrorKind::RateLimited,
        "locked for a while after too many failures"
    );
    w.advance(throttle.lock_for);
    let (session, _) = w.session("eva@example.org").await;
    assert_eq!(session.account, eva);
    assert_eq!(
        error(w.sign_in("eva@example.org", "a wrong password", None).await).0,
        ErrorKind::Unauthenticated,
        "a success starts the count again"
    );

    let address = Some("203.0.113.7");
    for i in 0..throttle.address_lock_after {
        drop(
            w.sign_in(&format!("guess{i}@example.org"), "a wrong password", address)
                .await,
        );
    }
    assert_eq!(
        error(w.sign_in("eva@example.org", PASSWORD, address).await).0,
        ErrorKind::RateLimited,
        "an address with too many failures is locked for every name"
    );
}

#[tokio::test]
async fn sessions_are_checked_on_every_request_and_revoked_centrally() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let (session, token) = w.session("ana@example.org").await;
    assert_eq!(w.identity.authenticate(&token).await.unwrap().id, session.id);
    let (_, other_token) = w.session("ana@example.org").await;
    assert_eq!(w.identity.sessions(&session).await.unwrap().len(), 2);
    let api = w
        .identity
        .create_own_token(
            &session,
            TokenRequest {
                name: "calendar".to_owned(),
                scopes: vec![TokenScope {
                    permission: TOKEN_MANAGE_OWN.to_owned(),
                    scope: w.campus,
                }],
                expires_at: None,
            },
        )
        .await
        .unwrap();

    w.identity
        .suspend(&Actor::Principal(w.manager), ana, "investigation", None)
        .await
        .unwrap();
    let invalid = (ErrorKind::Unauthenticated, "identity.session_invalid".to_owned());
    assert_eq!(error(w.identity.authenticate(&token).await), invalid);
    assert_eq!(error(w.identity.authenticate(&other_token).await), invalid);
    assert_eq!(w.count("session", ana).await, 0);
    assert_eq!(
        error(w.identity.authenticate_token(&api.token).await).0,
        ErrorKind::Unauthenticated
    );

    w.identity
        .reactivate(&Actor::Principal(w.manager), ana, "cleared", None)
        .await
        .unwrap();
    let (session, token) = w.session("ana@example.org").await;
    w.identity
        .change_password(&session, PASSWORD, "an even longer and quieter river")
        .await
        .unwrap();
    assert_eq!(error(w.identity.authenticate(&token).await), invalid);
    assert_eq!(
        w.courier.last_to("ana@example.org").map(|m| m.notice),
        Some(Notice::PasswordChanged)
    );

    let (_, token) = match w
        .sign_in("ana@example.org", "an even longer and quieter river", None)
        .await
        .unwrap()
    {
        SignInOutcome::SignedIn(new) => (new.session, new.token),
        SignInOutcome::SecondFactorRequired { .. } => unreachable!(),
    };
    w.advance(time::Duration::hours(9));
    assert_eq!(
        error(w.identity.authenticate(&token).await),
        invalid,
        "idle too long"
    );
    w.identity.sign_out("not a token").await.unwrap();
}

#[tokio::test]
async fn sensitive_operations_need_a_recent_strong_authentication() {
    let w = world().await;
    w.person("ana@example.org").await;
    let (session, token) = w.session("ana@example.org").await;
    w.advance(time::Duration::minutes(16));
    let session = w.identity.authenticate(&token).await.unwrap_or(session);
    assert_eq!(
        error(
            w.identity
                .change_password(&session, PASSWORD, "a brand new long river")
                .await
        ),
        (
            ErrorKind::PermissionDenied,
            "identity.step_up_required".to_owned()
        )
    );
    assert_eq!(
        error(w.identity.step_up(&session, "a wrong password").await).0,
        ErrorKind::Unauthenticated
    );
    let session = w.identity.step_up(&session, PASSWORD).await.unwrap();
    w.identity
        .change_password(&session, PASSWORD, "a brand new long river")
        .await
        .unwrap();
}

#[tokio::test]
async fn api_tokens_are_scoped_expiring_and_hashed() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let (session, _) = w.session("ana@example.org").await;
    let request = |permission: &str, expires_at: Option<Timestamp>| TokenRequest {
        name: "calendar".to_owned(),
        scopes: vec![TokenScope {
            permission: permission.to_owned(),
            scope: w.campus,
        }],
        expires_at,
    };
    let not_held = w
        .identity
        .create_own_token(&session, request(ACCOUNT_READ, None))
        .await
        .unwrap_err();
    assert!(not_held.fields().iter().any(|f| f.rule == "not_held"));
    let too_long = Timestamp::from_offset_date_time(w.identity_now() + time::Duration::days(400));
    let out_of_range = w
        .identity
        .create_own_token(&session, request(TOKEN_MANAGE_OWN, Some(too_long)))
        .await
        .unwrap_err();
    assert!(out_of_range.fields().iter().any(|f| f.field == "expires_at"));

    let token = w
        .identity
        .create_own_token(&session, request(TOKEN_MANAGE_OWN, None))
        .await
        .unwrap();
    assert!(token.token.starts_with(&format!("kmp_{}_", token.prefix)));
    assert_eq!(
        token.expires_at.as_offset_date_time() - w.identity_now(),
        time::Duration::days(90)
    );
    let stored: Vec<u8> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select hash from {} where id = $1",
        w.table("api_token")
    )))
    .bind(token.id.as_uuid())
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_ne!(stored.as_slice(), token.token.as_bytes());

    let auth = w.identity.authenticate_token(&token.token).await.unwrap();
    assert_eq!(auth.account, ana);
    assert!(
        w.identity
            .token_may(&auth, TOKEN_MANAGE_OWN, w.campus)
            .await
            .unwrap()
    );
    assert!(!w.identity.token_may(&auth, ACCOUNT_READ, w.campus).await.unwrap());
    let listed = w.identity.tokens(&session).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].prefix, token.prefix);

    let forged = format!("kmp_{}_{}", token.prefix, "x".repeat(43));
    assert_eq!(
        error(w.identity.authenticate_token(&forged).await).0,
        ErrorKind::Unauthenticated
    );
    w.identity.revoke_own_token(&session, token.id).await.unwrap();
    assert_eq!(
        error(w.identity.authenticate_token(&token.token).await).0,
        ErrorKind::Unauthenticated
    );

    let short = w
        .identity
        .create_own_token(
            &session,
            request(
                TOKEN_MANAGE_OWN,
                Some(Timestamp::from_offset_date_time(
                    w.identity_now() + time::Duration::days(1),
                )),
            ),
        )
        .await
        .unwrap();
    w.advance(time::Duration::days(2));
    assert_eq!(
        error(w.identity.authenticate_token(&short.token).await).0,
        ErrorKind::Unauthenticated
    );
}

#[tokio::test]
async fn deletion_erases_personal_data_and_leaves_a_tombstone() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    w.session("ana@example.org").await;
    w.identity
        .schedule_deletion(&w.admin, ana, "asked by the person", None)
        .await
        .unwrap();
    w.advance(time::Duration::days(15));
    assert_eq!(w.identity.run_due_deletions().await.unwrap(), 1);
    let tombstone = w.account(ana).await;
    assert_eq!(tombstone.status, AccountStatus::Deleted);
    assert_eq!((tombstone.username, tombstone.email), (None, None));
    assert!(tombstone.pseudonym.is_some_and(|p| p.starts_with("deleted-")));
    for table in ["password", "session", "sign_in_attempt", "one_time_token"] {
        assert_eq!(w.count(table, ana).await, 0, "{table}");
    }
    assert_eq!(
        error(w.sign_in("ana@example.org", PASSWORD, None).await).0,
        ErrorKind::Unauthenticated
    );
    let again = w.person("ana@example.org").await;
    assert_ne!(
        again, ana,
        "the email is free for a new account; the tombstone keeps its id"
    );
}

#[tokio::test]
async fn deletion_waits_for_the_grace_period_and_can_be_cancelled() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    w.identity
        .schedule_deletion(&w.admin, ana, "asked by the person", None)
        .await
        .unwrap();
    let scheduled = w.account(ana).await;
    assert_eq!(scheduled.status, AccountStatus::Suspended);
    assert!(scheduled.deletion_due_at.is_some());
    assert_eq!(
        error(w.sign_in("ana@example.org", PASSWORD, None).await).0,
        ErrorKind::Unauthenticated
    );
    w.advance(time::Duration::days(13));
    assert_eq!(w.identity.run_due_deletions().await.unwrap(), 0);
    w.identity.cancel_deletion(&w.admin, ana, None).await.unwrap();
    w.advance(time::Duration::days(2));
    assert_eq!(w.identity.run_due_deletions().await.unwrap(), 0);
    assert_eq!(w.account(ana).await.status, AccountStatus::Suspended);
    w.identity
        .reactivate(&w.admin, ana, "changed their mind", None)
        .await
        .unwrap();
    w.session("ana@example.org").await;
}

#[tokio::test]
async fn service_accounts_authenticate_only_with_api_tokens() {
    let w = world().await;
    let service = w
        .identity
        .create_service_account(&w.admin, w.campus, "Grades-Sync")
        .await
        .unwrap();
    assert_eq!(w.count("password", service).await, 0);
    assert_eq!(
        error(w.sign_in("grades-sync", PASSWORD, None).await).0,
        ErrorKind::Unauthenticated
    );
    w.access
        .grant(&w.admin, w.roles["manager"], w.campus, service, None)
        .await
        .unwrap();
    let request = TokenRequest {
        name: "grades".to_owned(),
        scopes: vec![TokenScope {
            permission: ACCOUNT_READ.to_owned(),
            scope: w.campus,
        }],
        expires_at: None,
    };
    let manager = Actor::Principal(w.manager);
    assert_eq!(
        error(
            w.identity
                .create_token_for(&manager, service, request.clone())
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
    let token = w
        .identity
        .create_token_for(&w.admin, service, request)
        .await
        .unwrap();
    let auth = w.identity.authenticate_token(&token.token).await.unwrap();
    assert_eq!(auth.account, service);
    assert!(w.identity.token_may(&auth, ACCOUNT_READ, w.campus).await.unwrap());
    w.identity.revoke_token(&w.admin, token.id).await.unwrap();
    assert_eq!(
        error(w.identity.authenticate_token(&token.token).await).0,
        ErrorKind::Unauthenticated
    );
}

/// The current code of an authenticator app with this base32 secret.
fn current_code(w: &World, secret: &str) -> String {
    let bytes = totp::base32_decode(secret).unwrap();
    totp::code(
        &bytes,
        totp::step_of(w.identity_now().unix_timestamp()),
        totp::DIGITS,
    )
}

/// Signs in with [`PASSWORD`], expecting a second factor to be asked.
async fn challenge(w: &World, name: &str) -> (String, Vec<FactorKind>) {
    match w.sign_in(name, PASSWORD, None).await.unwrap() {
        SignInOutcome::SecondFactorRequired { challenge, methods } => (challenge, methods),
        SignInOutcome::SignedIn(_) => panic!("a second factor was expected"),
    }
}

#[tokio::test]
async fn second_factors_are_required_once_set_up() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let (session, _) = w.session("ana@example.org").await;
    let client = Client::default();

    let setup = w.identity.new_totp_secret(&session).await.unwrap();
    assert!(
        setup
            .uri
            .starts_with("otpauth://totp/Kampus:ana%40example.org?secret=")
    );
    assert_eq!(
        error(
            w.identity
                .confirm_totp(&session, "phone", &setup.secret, "000000")
                .await
        )
        .1,
        "identity.totp_code_wrong"
    );
    let added = w
        .identity
        .confirm_totp(&session, "phone", &setup.secret, &current_code(&w, &setup.secret))
        .await
        .unwrap();
    let codes = added
        .recovery_codes
        .expect("the first factor comes with recovery codes");
    assert_eq!(codes.len(), 10);
    assert_eq!(
        w.courier.last_to("ana@example.org").map(|m| m.notice),
        Some(Notice::SecondFactorAdded)
    );
    let sealed: Vec<u8> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select secret_encrypted from {} where account_id = $1",
        w.table("second_factor")
    )))
    .bind(ana.as_uuid())
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_ne!(
        sealed,
        totp::base32_decode(&setup.secret).unwrap(),
        "secrets are sealed"
    );

    let (first, methods) = challenge(&w, "ana@example.org").await;
    assert_eq!(methods, vec![FactorKind::Totp]);
    let used = current_code(&w, &setup.secret);
    assert_eq!(
        error(
            w.identity
                .complete_sign_in(&first, Proof::Totp(used.clone()), &client)
                .await
        ),
        (
            ErrorKind::Unauthenticated,
            "identity.second_factor_failed".to_owned()
        ),
        "the code that confirmed the app cannot be used again"
    );
    assert_eq!(
        error(
            w.identity
                .complete_sign_in(&first, Proof::Totp(used), &client)
                .await
        )
        .1,
        "identity.token_invalid",
        "a challenge works once"
    );
    w.advance(time::Duration::seconds(30));
    let (second, _) = challenge(&w, "ana@example.org").await;
    let completed = w
        .identity
        .complete_sign_in(&second, Proof::Totp(current_code(&w, &setup.secret)), &client)
        .await
        .unwrap();
    assert_eq!(completed.session.session.mfa_level, MfaLevel::SecondFactor);

    let (third, _) = challenge(&w, "ana@example.org").await;
    w.identity
        .complete_sign_in(&third, Proof::RecoveryCode(codes[0].to_uppercase()), &client)
        .await
        .unwrap();
    let (fourth, _) = challenge(&w, "ana@example.org").await;
    assert_eq!(
        error(
            w.identity
                .complete_sign_in(&fourth, Proof::RecoveryCode(codes[0].clone()), &client)
                .await
        )
        .1,
        "identity.second_factor_failed",
        "a recovery code works once"
    );

    let session = completed.session.session;
    let email = w.identity.add_email_factor(&session, "mail").await.unwrap();
    assert!(email.recovery_codes.is_none(), "the recovery codes stay");
    let (fifth, methods) = challenge(&w, "ana@example.org").await;
    assert_eq!(methods, vec![FactorKind::Email, FactorKind::Totp]);
    w.identity.send_email_code(&fifth).await.unwrap();
    let Some(Notice::EmailCode { code }) = w.courier.last_to("ana@example.org").map(|m| m.notice) else {
        panic!("an email code was sent");
    };
    w.identity
        .complete_sign_in(&fifth, Proof::EmailCode(code), &client)
        .await
        .unwrap();

    let factors = w.identity.second_factors(&session).await.unwrap();
    assert_eq!(factors.len(), 2);
    for factor in factors {
        w.identity
            .remove_second_factor(&session, factor.id)
            .await
            .unwrap();
    }
    assert_eq!(w.count("recovery_code", ana).await, 0);
    w.session("ana@example.org").await;
}

#[tokio::test]
async fn a_role_can_require_a_second_factor() {
    let w = world().await;
    let bea = w.person("bea@example.org").await;
    w.access
        .set_role_second_factor(&w.admin, w.roles["manager"], true)
        .await
        .unwrap();
    assert_eq!(
        error(
            w.access
                .set_role_second_factor(&Actor::Principal(bea), w.roles["manager"], false)
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
    w.session("bea@example.org").await;
    w.access
        .grant(&w.admin, w.roles["manager"], w.campus, bea, None)
        .await
        .unwrap();
    let (challenge_token, methods) = challenge(&w, "bea@example.org").await;
    assert!(methods.is_empty(), "the account must set up an app first");
    let secret = totp::base32_encode(&[9_u8; 20]);
    let completed = w
        .identity
        .complete_sign_in(
            &challenge_token,
            Proof::EnrolTotp {
                label: "phone".to_owned(),
                secret: secret.clone(),
                code: current_code(&w, &secret),
            },
            &Client::default(),
        )
        .await
        .unwrap();
    assert_eq!(completed.recovery_codes.map(|c| c.len()), Some(10));
    let session = completed.session.session;
    let factor = w.identity.second_factors(&session).await.unwrap()[0].id;
    assert_eq!(
        error(w.identity.remove_second_factor(&session, factor).await),
        (ErrorKind::Conflict, "identity.second_factor_required".to_owned())
    );
}

#[tokio::test]
async fn recovery_checks_the_second_factor_first_and_never_signs_in() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let (session, old_token) = w.session("ana@example.org").await;
    let setup = w.identity.new_totp_secret(&session).await.unwrap();
    w.identity
        .confirm_totp(&session, "phone", &setup.secret, &current_code(&w, &setup.secret))
        .await
        .unwrap();
    let recovery_link = |w: &World| match w.courier.last_to("ana@example.org").map(|m| m.notice) {
        Some(Notice::Recovery { token }) => token,
        other => panic!("a recovery link was expected: {other:?}"),
    };

    let before = w.courier.sent().len();
    w.identity
        .start_recovery(w.campus, "nobody@example.org")
        .await
        .unwrap();
    w.identity.start_recovery(w.campus, "not an email").await.unwrap();
    assert_eq!(
        w.courier.sent().len(),
        before,
        "unknown emails get the same answer and no message"
    );

    w.identity
        .start_recovery(w.campus, "ANA@example.org")
        .await
        .unwrap();
    let first = recovery_link(&w);
    assert_eq!(
        w.identity.recovery_needs(&first).await.unwrap().methods,
        vec![FactorKind::Totp]
    );
    let new_password = "a recovered and quiet river";
    assert_eq!(
        error(w.identity.complete_recovery(&first, None, new_password).await).1,
        "identity.second_factor_failed"
    );
    assert_eq!(
        error(w.identity.complete_recovery(&first, None, new_password).await).1,
        "identity.token_invalid",
        "a failed recovery uses up its link"
    );
    w.identity.authenticate(&old_token).await.unwrap();

    w.advance(time::Duration::seconds(30));
    w.identity
        .start_recovery(w.campus, "ana@example.org")
        .await
        .unwrap();
    let second = recovery_link(&w);
    w.identity
        .complete_recovery(
            &second,
            Some(Proof::Totp(current_code(&w, &setup.secret))),
            new_password,
        )
        .await
        .unwrap();
    assert_eq!(
        w.courier.last_to("ana@example.org").map(|m| m.notice),
        Some(Notice::RecoveryUsed)
    );
    assert_eq!(
        error(w.identity.authenticate(&old_token).await).0,
        ErrorKind::Unauthenticated
    );
    assert_eq!(w.count("session", ana).await, 0, "recovery does not sign in");
    assert!(matches!(
        w.sign_in("ana@example.org", new_password, None).await.unwrap(),
        SignInOutcome::SecondFactorRequired { .. }
    ));

    w.identity
        .start_recovery(w.campus, "ana@example.org")
        .await
        .unwrap();
    let links = w
        .courier
        .sent()
        .iter()
        .filter(|m| matches!(m.notice, Notice::Recovery { .. }))
        .count();
    assert_eq!(links, 3, "at most three links an hour");
}

/// Where the browser comes back from the provider.
const REDIRECT: &str = "http://localhost/callback";

/// Configures a test provider for the campus: its issuer gives the claims the test needs.
async fn provider(w: &World, issuer: &str, linking_policy: LinkingPolicy) -> ProviderId {
    let base = std::env::var("KAMPUS_TEST_OIDC_URL").expect("KAMPUS_TEST_OIDC_URL is set");
    let issuer = format!("{base}/{issuer}");
    w.identity
        .add_provider(
            &w.admin,
            w.campus,
            ProviderConfig {
                issuer,
                client: "kampus".to_owned(),
                client_secret: "provider secret".to_owned(),
                linking_policy,
            },
        )
        .await
        .unwrap()
}

/// Signs in at the provider as its browser would, and returns the flow, the code and the state.
async fn at_provider(w: &World, provider: ProviderId) -> (String, String, String) {
    let start = w.identity.oidc_start(provider, REDIRECT).await.unwrap();
    let url = Url::parse(&start.authorize_url).unwrap();
    let query: BTreeMap<String, String> = url.query_pairs().into_owned().collect();
    assert_eq!(query["code_challenge_method"], "S256");
    assert!(query.contains_key("nonce"));
    let browser = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = browser.get(url).send().await.unwrap();
    let location = Url::parse(response.headers()["location"].to_str().unwrap()).unwrap();
    assert!(location.as_str().starts_with(REDIRECT));
    let back: BTreeMap<String, String> = location.query_pairs().into_owned().collect();
    (start.flow, back["code"].clone(), back["state"].clone())
}

/// Signs in through the provider.
async fn through_provider(w: &World, provider: ProviderId) -> Result<OidcOutcome> {
    let (flow, code, state) = at_provider(w, provider).await;
    w.identity
        .oidc_callback(&flow, &code, &state, &Client::default())
        .await
}

#[tokio::test]
async fn a_verified_email_links_by_policy_and_then_by_subject() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let google = provider(&w, "verified", LinkingPolicy::VerifiedEmail).await;
    assert_eq!(w.identity.providers(w.campus).await.unwrap().len(), 1);
    assert_eq!(
        error(provider_again(&w).await).0,
        ErrorKind::Conflict,
        "one provider per issuer in a campus"
    );

    let (flow, code, state) = at_provider(&w, google).await;
    assert_eq!(
        error(
            w.identity
                .oidc_callback(&flow, &code, "another state", &Client::default())
                .await
        )
        .1,
        "identity.oidc_flow_invalid"
    );
    let mut tampered = flow.clone();
    tampered.replace_range(10..11, if &flow[10..11] == "A" { "B" } else { "A" });
    assert_eq!(
        error(
            w.identity
                .oidc_callback(&tampered, &code, &state, &Client::default())
                .await
        )
        .1,
        "identity.oidc_flow_invalid"
    );
    let outcome = w
        .identity
        .oidc_callback(&flow, &code, &state, &Client::default())
        .await
        .unwrap();
    let OidcOutcome::SignedIn(SignInOutcome::SignedIn(new)) = outcome else {
        panic!("signed in through the provider: {outcome:?}");
    };
    assert_eq!(new.session.account, ana);
    assert_eq!(w.count("external_identity", ana).await, 1);

    let OidcOutcome::SignedIn(SignInOutcome::SignedIn(again)) = through_provider(&w, google).await.unwrap()
    else {
        panic!("the link by subject signs in again");
    };
    assert_eq!(again.session.account, ana);

    let setup = w.identity.new_totp_secret(&new.session).await.unwrap();
    w.identity
        .confirm_totp(
            &new.session,
            "phone",
            &setup.secret,
            &current_code(&w, &setup.secret),
        )
        .await
        .unwrap();
    assert!(
        matches!(
            through_provider(&w, google).await.unwrap(),
            OidcOutcome::SignedIn(SignInOutcome::SecondFactorRequired { .. })
        ),
        "the provider goes through the same pipeline, second factor included"
    );

    w.identity
        .suspend(&Actor::Principal(w.manager), ana, "paused", None)
        .await
        .unwrap();
    assert_eq!(
        error(through_provider(&w, google).await),
        (
            ErrorKind::Unauthenticated,
            "identity.external_identity_not_linked".to_owned()
        )
    );
}

/// Adds the same issuer a second time.
async fn provider_again(w: &World) -> Result<ProviderId> {
    w.identity
        .add_provider(
            &w.admin,
            w.campus,
            ProviderConfig {
                issuer: format!("{}/verified", std::env::var("KAMPUS_TEST_OIDC_URL").unwrap()),
                client: "kampus".to_owned(),
                client_secret: "provider secret".to_owned(),
                linking_policy: LinkingPolicy::SubjectOnly,
            },
        )
        .await
}

#[tokio::test]
async fn an_unverified_or_unknown_email_never_links() {
    let not_linked = (
        ErrorKind::Unauthenticated,
        "identity.external_identity_not_linked".to_owned(),
    );
    for (issuer, policy) in [
        ("unverified", LinkingPolicy::VerifiedEmail),
        ("stranger", LinkingPolicy::VerifiedEmail),
        ("verified", LinkingPolicy::SubjectOnly),
    ] {
        let w = world().await;
        let ana = w.person("ana@example.org").await;
        let provider = provider(&w, issuer, policy).await;
        assert_eq!(
            error(through_provider(&w, provider).await),
            not_linked,
            "{issuer}"
        );
        assert_eq!(w.count("external_identity", ana).await, 0);
    }
    let w = world().await;
    let manager = Actor::Principal(w.manager);
    assert_eq!(
        error(
            w.identity
                .add_provider(
                    &manager,
                    w.campus,
                    ProviderConfig {
                        issuer: "https://accounts.example.org".to_owned(),
                        client: "kampus".to_owned(),
                        client_secret: "secret".to_owned(),
                        linking_policy: LinkingPolicy::Confirm,
                    },
                )
                .await
        )
        .0,
        ErrorKind::PermissionDenied
    );
}

#[tokio::test]
async fn the_owner_can_confirm_a_link() {
    let w = world().await;
    let ana = w.person("ana@example.org").await;
    let provider = provider(&w, "verified", LinkingPolicy::Confirm).await;
    let OidcOutcome::ConfirmLink { link } = through_provider(&w, provider).await.unwrap() else {
        panic!("the campus asks the owner to confirm");
    };
    assert_eq!(
        w.count("external_identity", ana).await,
        0,
        "nothing is linked before the owner confirms"
    );
    let (session, _) = w.session("ana@example.org").await;
    w.identity.confirm_link(&session, &link).await.unwrap();
    assert_eq!(
        w.courier.last_to("ana@example.org").map(|m| m.notice),
        Some(Notice::IdentityLinked)
    );
    let OidcOutcome::SignedIn(SignInOutcome::SignedIn(new)) = through_provider(&w, provider).await.unwrap()
    else {
        panic!("the confirmed link signs in");
    };
    assert_eq!(new.session.account, ana);

    let bea = w.person("bea@example.org").await;
    let (bea_session, _) = w.session("bea@example.org").await;
    assert_eq!(bea_session.account, bea);
    assert_eq!(
        error(w.identity.confirm_link(&bea_session, &link).await),
        (
            ErrorKind::Conflict,
            "identity.external_identity_linked".to_owned()
        ),
        "an identity links to one account"
    );
}
