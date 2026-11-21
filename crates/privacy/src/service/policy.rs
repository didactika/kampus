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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Policies: a campus publishes them in versions, and each person accepts the version in force. What a person
//! accepted is part of their personal data.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_module::{PrivacyFuture, PrivacyHandlerDeclaration, PrivacyRun};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::permissions::POLICY_MANAGE;
use crate::{Policy, PolicyId, Privacy, events, sql};

/// The handlers of this module.
pub const HANDLERS: &[PrivacyHandlerDeclaration] = &[PrivacyHandlerDeclaration {
    code: "privacy.acceptances",
    purpose: "Which version of each policy the person accepted, and when",
    export,
    erase: keep,
}];

/// Keeps the acceptances: they show what the person agreed to, and point to the tombstone of the account.
fn keep(_run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move { Ok(json!({ "kept": "proof of acceptance" })) })
}

/// Exports the acceptances of the account of a request.
fn export(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        run.services
            .service::<Privacy>()?
            .acceptances_of(PrincipalId::from_uuid(run.account))
            .await
    })
}

/// The person behind an actor.
fn person(actor: &Actor) -> Result<PrincipalId> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Ok(*p),
        Actor::Visitor(_) => Err(Error::permission_denied("access.denied")),
    }
}

/// A policy from its row.
fn policy(row: &PgRow) -> Policy {
    Policy {
        id: PolicyId::from_uuid(row.get("id")),
        code: row.get("code"),
        title: row.get("title"),
        version: row.get("version"),
        body: row.get("body"),
        published_at: Timestamp::from_offset_date_time(row.get("published_at")),
        accepted_at: row
            .get::<Option<time::OffsetDateTime>, _>("accepted_at")
            .map(Timestamp::from_offset_date_time),
    }
}

impl Privacy {
    /// The query for the latest version of each policy of a campus (`$1`) with when an account (`$2`)
    /// accepted it, narrowed by `extra`.
    fn latest_sql(&self, extra: &str) -> String {
        format!(
            "select p.id, p.code, p.title, v.version, v.body, v.published_at, a.accepted_at
             from {p} p
             join {v} v on v.policy_id = p.id
                 and v.version = (select max(version) from {v} where policy_id = p.id)
             left join {a} a on a.policy_id = p.id and a.version = v.version and a.account_id = $2
             where p.campus_id = $1 {extra} order by p.code",
            p = self.table("policy"),
            v = self.table("policy_version"),
            a = self.table("acceptance"),
        )
    }

    /// Publishes a version of a policy of the campus, creating the policy the first time its code is used.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `privacy.policy.manage`, `validation` for an empty code, title or body.
    pub async fn publish_policy(&self, actor: &Actor, code: &str, title: &str, body: &str) -> Result<Policy> {
        let by = person(actor)?;
        let campus = self.access.principal_campus(by).await?;
        if !self.access.check(actor, POLICY_MANAGE, campus).await?.allowed {
            return Err(Error::permission_denied("access.denied").with_param("permission", POLICY_MANAGE));
        }
        let code = code.trim().to_lowercase();
        if code.is_empty() || title.trim().is_empty() || body.trim().is_empty() {
            return Err(Error::validation(
                "privacy.policy_incomplete",
                vec![FieldError::new("code, title and body", "required")],
            ));
        }
        let mut tx = self.db.begin().await?;
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {} (id, campus_id, code, title) values ($1, $2, $3, $4)
             on conflict (campus_id, code) do update set title = excluded.title returning id",
            self.table("policy")
        )))
        .bind(PolicyId::generate().as_uuid())
        .bind(campus.as_uuid())
        .bind(&code)
        .bind(title.trim())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let version: i32 = sqlx::query_scalar(sql(format!(
            "insert into {v} (policy_id, version, body, published_by)
             select $1, coalesce(max(version), 0) + 1, $2, $3 from {v} where policy_id = $1
             returning version",
            v = self.table("policy_version")
        )))
        .bind(id)
        .bind(body)
        .bind(by.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut event = kampus_events::Publish::new(
            events::POLICY_PUBLISHED,
            Some(id),
            json!({ "policy": id, "code": code, "version": version }),
        );
        event.actor = Some(by.as_uuid());
        event.summary = Some(format!("{} {code} v{version}", events::POLICY_PUBLISHED));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.policy_of(by, &code).await
    }

    /// A policy of the campus of the person, in its latest version.
    async fn policy_of(&self, account: PrincipalId, code: &str) -> Result<Policy> {
        let campus = self.access.principal_campus(account).await?;
        sqlx::query(sql(self.latest_sql("and p.code = $3")))
            .bind(campus.as_uuid())
            .bind(account.as_uuid())
            .bind(code)
            .fetch_optional(self.db.pool())
            .await
            .map_err(kampus_db::classify)?
            .map(|row| policy(&row))
            .ok_or_else(|| Error::not_found("privacy.policy_not_found"))
    }

    /// The policies of the campus of the person in their latest version, with when the person accepted each.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn policies(&self, actor: &Actor) -> Result<Vec<Policy>> {
        self.listed(actor, "").await
    }

    /// The policies the person has not accepted in their latest version.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn pending_policies(&self, actor: &Actor) -> Result<Vec<Policy>> {
        self.listed(actor, "and a.accepted_at is null").await
    }

    /// The latest versions, narrowed by a condition.
    async fn listed(&self, actor: &Actor, extra: &str) -> Result<Vec<Policy>> {
        let me = person(actor)?;
        let campus = self.access.principal_campus(me).await?;
        let rows = sqlx::query(sql(self.latest_sql(extra)))
            .bind(campus.as_uuid())
            .bind(me.as_uuid())
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(policy).collect())
    }

    /// Accepts a version of a policy of the campus; accepting again changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found` (`privacy.policy_not_found`) for an unknown policy or version, `conflict`
    /// (`privacy.policy_outdated`) for a version that is no longer the latest.
    pub async fn accept_policy(&self, actor: &Actor, code: &str, version: i32) -> Result<Policy> {
        let me = person(actor)?;
        let current = self.policy_of(me, &code.trim().to_lowercase()).await?;
        if version > current.version || version < 1 {
            return Err(Error::not_found("privacy.policy_not_found"));
        }
        if version < current.version {
            return Err(Error::conflict("privacy.policy_outdated"));
        }
        let mut tx = self.db.begin().await?;
        let inserted = sqlx::query(sql(format!(
            "insert into {} (policy_id, version, account_id) values ($1, $2, $3) on conflict do nothing",
            self.table("acceptance")
        )))
        .bind(current.id.as_uuid())
        .bind(version)
        .bind(me.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if inserted > 0 {
            let mut event = kampus_events::Publish::new(
                events::POLICY_ACCEPTED,
                Some(current.id.as_uuid()),
                json!({ "policy": current.id, "version": version, "account": me }),
            );
            event.actor = Some(me.as_uuid());
            event.summary = Some(format!("{} {} v{version}", events::POLICY_ACCEPTED, current.code));
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, event)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.policy_of(me, &current.code).await
    }

    /// Everything this module keeps about an account: what they accepted and when.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn acceptances_of(&self, account: PrincipalId) -> Result<Value> {
        let rows: Value = sqlx::query_scalar(sql(format!(
            "select coalesce(jsonb_agg(jsonb_build_object(
                 'policy', p.code, 'version', a.version, 'accepted_at', a.accepted_at)
                 order by p.code, a.version), '[]'::jsonb)
             from {} a join {} p on p.id = a.policy_id where a.account_id = $1",
            self.table("acceptance"),
            self.table("policy")
        )))
        .bind(account.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(json!({ "acceptance": rows }))
    }
}
