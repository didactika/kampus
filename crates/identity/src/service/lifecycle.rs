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

//! The account lifecycle: invitations, service accounts, suspension, reactivation and
//! deletion with a grace period. Each transition needs its permission in the account's campus,
//! honours `If-Match`, is recorded, and keeps the principal active only while the account is.
//! Suspending or deleting revokes every session and token at once.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, ErrorKind, Result};
use sqlx::PgConnection;

use crate::permissions::{ACCOUNT_DELETE, ACCOUNT_INVITE, ACCOUNT_SUSPEND};
use crate::service::account::{NewAccount, normalise_email, normalise_username};
use crate::service::one_time::Purpose;
use crate::{Account, AccountId, AccountKind, AccountStatus, Identity, Message, Notice, later, sql};

/// The principal behind an actor, if it is one.
pub(crate) const fn principal(actor: &Actor) -> Option<AccountId> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(*p),
        Actor::Visitor(_) => None,
    }
}

impl Identity {
    /// Moves an account to another status and keeps everything that follows from it in step.
    pub(crate) async fn transition_in(
        &self,
        conn: &mut PgConnection,
        account: &Account,
        to: AccountStatus,
        reason: Option<&str>,
        actor: Option<AccountId>,
    ) -> Result<()> {
        if !account.status.may_become(to) {
            return Err(Error::conflict("identity.transition_not_allowed")
                .with_param("from", account.status.as_str())
                .with_param("to", to.as_str()));
        }
        sqlx::query(sql(format!(
            "update {} set status = $2, status_reason = $3, status_changed_at = $4, status_changed_by = $5,
             deletion_due_at = case when $2 = 'active' then null else deletion_due_at end, version = version + 1
             where id = $1",
            self.table("account")
        )))
        .bind(account.id.as_uuid())
        .bind(to.as_str())
        .bind(reason)
        .bind(self.now().as_offset_date_time())
        .bind(actor.map(|a| a.as_uuid()))
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        self.record_transition_in(conn, account.id, Some(account.status), to, reason, actor)
            .await?;
        let active = to == AccountStatus::Active;
        self.access
            .set_principal_active_in(conn, account.id, active)
            .await?;
        if !active {
            self.revoke_everything_in(conn, account.id).await?;
        }
        let first_activation = matches!(
            account.status,
            AccountStatus::Invited | AccountStatus::PendingConfirmation
        );
        if active && first_activation && account.kind == AccountKind::Person {
            self.grant_membership_in(conn, account).await?;
        }
        Ok(())
    }

    /// Locks an account for a manager's change: checks the permission in its campus and `If-Match`.
    async fn lock_for_change(
        &self,
        conn: &mut PgConnection,
        actor: &Actor,
        permission: &str,
        id: AccountId,
        expected_version: Option<i32>,
    ) -> Result<Account> {
        let account = self.lock_account_in(conn, id).await?;
        self.access.require(actor, permission, account.campus).await?;
        if principal(actor) == Some(id) {
            return Err(Error::conflict("identity.own_account"));
        }
        if expected_version.is_some_and(|v| v != account.version) {
            return Err(
                Error::new(ErrorKind::PreconditionFailed, "identity.account_changed")
                    .with_param("version", account.version.to_string()),
            );
        }
        Ok(account)
    }

    /// Invites a person to a campus: the account waits for the invitation link, which sets the
    /// password. A manager may know that an email has an account, so taken names are a conflict.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `identity.account.invite`, `validation` for an invalid email or
    /// username, `conflict` when the email or username has an account.
    pub async fn invite(
        &self,
        actor: &Actor,
        campus: ScopeId,
        email: &str,
        username: Option<&str>,
    ) -> Result<AccountId> {
        self.access.require(actor, ACCOUNT_INVITE, campus).await?;
        let email = normalise_email(email)?;
        let username = match username {
            Some(username) => normalise_username(username)?,
            None => email.clone(),
        };
        let policy = self.policies.policy(campus).await?;
        let mut tx = self.db.begin().await?;
        let account = self
            .insert_account_in(
                &mut tx,
                NewAccount {
                    campus,
                    kind: AccountKind::Person,
                    username: &username,
                    email: Some(&email),
                    status: AccountStatus::Invited,
                    actor: principal(actor),
                },
            )
            .await
            .map_err(taken)?;
        let token = self
            .issue_in(
                &mut tx,
                account.id,
                Purpose::Invitation,
                policy.invitation_lifetime,
            )
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.deliver(Message {
            campus,
            account: account.id,
            email,
            notice: Notice::Invitation { token },
        })
        .await;
        Ok(account.id)
    }

    /// Accepts an invitation with the link's token and the password the person chooses.
    ///
    /// # Errors
    ///
    /// `validation` for an invalid token or a rejected password.
    pub async fn accept_invitation(&self, token: &str, password: &str) -> Result<AccountId> {
        self.activate_with(Purpose::Invitation, AccountStatus::Invited, token, password)
            .await
    }

    /// Creates a service account: active at once, without password or second factor; it acts
    /// only through API tokens and holds only the roles granted to it.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `identity.account.invite`, `validation` for an invalid username,
    /// `conflict` when it is taken.
    pub async fn create_service_account(
        &self,
        actor: &Actor,
        campus: ScopeId,
        username: &str,
    ) -> Result<AccountId> {
        self.access.require(actor, ACCOUNT_INVITE, campus).await?;
        let username = normalise_username(username)?;
        let mut tx = self.db.begin().await?;
        let account = self
            .insert_account_in(
                &mut tx,
                NewAccount {
                    campus,
                    kind: AccountKind::Service,
                    username: &username,
                    email: None,
                    status: AccountStatus::Active,
                    actor: principal(actor),
                },
            )
            .await
            .map_err(taken)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(account.id)
    }

    /// Suspends an account with a reason: it cannot sign in, and its sessions and tokens end.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `identity.account.suspend`, `precondition_failed` for a stale version,
    /// `conflict` for a transition the lifecycle does not allow or for one's own account.
    pub async fn suspend(
        &self,
        actor: &Actor,
        id: AccountId,
        reason: &str,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let account = self
            .lock_for_change(&mut tx, actor, ACCOUNT_SUSPEND, id, expected_version)
            .await?;
        self.transition_in(
            &mut tx,
            &account,
            AccountStatus::Suspended,
            Some(reason),
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.tell(&account, Notice::Suspended).await;
        Ok(())
    }

    /// Reactivates a suspended account, cancelling a scheduled deletion.
    ///
    /// # Errors
    ///
    /// As [`Identity::suspend`].
    pub async fn reactivate(
        &self,
        actor: &Actor,
        id: AccountId,
        reason: &str,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let account = self
            .lock_for_change(&mut tx, actor, ACCOUNT_SUSPEND, id, expected_version)
            .await?;
        if account.status != AccountStatus::Suspended {
            return Err(Error::conflict("identity.transition_not_allowed")
                .with_param("from", account.status.as_str())
                .with_param("to", AccountStatus::Active.as_str()));
        }
        self.transition_in(
            &mut tx,
            &account,
            AccountStatus::Active,
            Some(reason),
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Schedules the deletion of an account: it is suspended now and deleted when the grace
    /// period of its campus ends, unless the deletion is cancelled or the account reactivated.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `identity.account.delete`, otherwise as [`Identity::suspend`].
    pub async fn schedule_deletion(
        &self,
        actor: &Actor,
        id: AccountId,
        reason: &str,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let account = self
            .lock_for_change(&mut tx, actor, ACCOUNT_DELETE, id, expected_version)
            .await?;
        if account.deletion_due_at.is_some() {
            return Err(Error::conflict("identity.deletion_scheduled"));
        }
        if account.status != AccountStatus::Suspended {
            self.transition_in(
                &mut tx,
                &account,
                AccountStatus::Suspended,
                Some(reason),
                principal(actor),
            )
            .await?;
        }
        let policy = self.policies.policy(account.campus).await?;
        sqlx::query(sql(format!(
            "update {} set deletion_due_at = $2, status_reason = $3, version = version + 1 where id = $1",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .bind(later(self.now(), policy.deletion_grace).as_offset_date_time())
        .bind(reason)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.tell(&account, Notice::Suspended).await;
        Ok(())
    }

    /// Cancels a scheduled deletion; the account stays suspended until it is reactivated.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `identity.account.delete`, `conflict` when no deletion is scheduled,
    /// `precondition_failed` for a stale version.
    pub async fn cancel_deletion(
        &self,
        actor: &Actor,
        id: AccountId,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let account = self
            .lock_for_change(&mut tx, actor, ACCOUNT_DELETE, id, expected_version)
            .await?;
        if account.deletion_due_at.is_none() {
            return Err(Error::conflict("identity.deletion_not_scheduled"));
        }
        sqlx::query(sql(format!(
            "update {} set deletion_due_at = null, version = version + 1 where id = $1",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Tells the owner of an account, when it has an email.
    async fn tell(&self, account: &Account, notice: Notice) {
        if let Some(email) = account.email.clone() {
            self.deliver(Message {
                campus: account.campus,
                account: account.id,
                email,
                notice,
            })
            .await;
        }
    }
}

/// Turns the unique violation of a taken username or email into a conflict a manager can read.
fn taken(error: Error) -> Error {
    if error.code() == "db.unique_violation" {
        Error::conflict("identity.account_exists")
    } else {
        error
    }
}
