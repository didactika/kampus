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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a campus sets for its channels, and the devices of a person: the templates of email and push in each
//! language, the defaults of each type and channel (locked or not), and the push subscriptions of the
//! browsers and phones a person allows.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::person;
use crate::db::steps::SENT_CHANNELS;
use crate::permissions::{DEFAULTS_MANAGE, TEMPLATE_MANAGE};
use crate::{CHANNELS, Notify, sql};

/// The longest subject of a template, in characters.
const SUBJECT_LIMIT: usize = 200;

/// The longest body of a template, in characters.
const BODY_LIMIT: usize = 10_000;

/// A template of a campus for a type, a channel and a language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    /// The type.
    #[serde(rename = "type")]
    pub type_code: String,
    /// The channel, `email` or `push`.
    pub channel: String,
    /// The language, such as `es`.
    pub locale: String,
    /// The subject, or the title of a push notification, with placeholders like `{{summary}}`.
    pub subject: String,
    /// The text, with placeholders taken from the notification.
    pub body: String,
}

/// What a campus sets for a type on a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampusDefault {
    /// The type.
    #[serde(rename = "type")]
    pub type_code: String,
    /// The channel.
    pub channel: String,
    /// Whether people receive it.
    pub enabled: bool,
    /// Whether people may change it.
    #[serde(default)]
    pub locked: bool,
}

/// A browser or phone a person allowed to receive push notifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushSubscription {
    /// The subscription.
    #[serde(default)]
    pub id: Option<uuid::Uuid>,
    /// Where its push service receives messages for it.
    pub endpoint: String,
    /// Its public key, as the browser gives it.
    pub p256dh: String,
    /// Its authentication secret, as the browser gives it.
    pub auth: String,
}

impl Notify {
    /// Fails unless the actor holds a permission in a campus.
    async fn require_in(&self, actor: &Actor, permission: &str, campus: ScopeId) -> Result<()> {
        self.access().require(actor, permission, campus).await
    }

    /// Fails unless a type is declared.
    async fn require_type(&self, type_code: &str) -> Result<()> {
        let known: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where code = $1 and retired_at is null)",
            self.table("notification_type")
        )))
        .bind(type_code)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if known {
            Ok(())
        } else {
            Err(Error::not_found("notify.type_unknown"))
        }
    }

    /// Writes the template of a campus for a type, a channel and a language, replacing the one there was;
    /// needs `notify.template.manage` in the campus.
    ///
    /// # Errors
    ///
    /// `permission_denied`, `not_found` (`notify.type_unknown`), `validation` (`notify.template_invalid`).
    pub async fn set_template(&self, actor: &Actor, campus: ScopeId, template: Template) -> Result<Template> {
        self.require_in(actor, TEMPLATE_MANAGE, campus).await?;
        self.require_type(&template.type_code).await?;
        let mut problems = Vec::new();
        if !SENT_CHANNELS.contains(&template.channel.as_str()) {
            problems.push(FieldError::new("channel", "one_of"));
        }
        let locale = template.locale.trim().to_lowercase();
        if locale.is_empty()
            || locale.len() > 10
            || !locale.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
        {
            problems.push(FieldError::new("locale", "format"));
        }
        if template.subject.trim().is_empty() || template.subject.chars().count() > SUBJECT_LIMIT {
            problems.push(FieldError::new("subject", "length"));
        }
        if template.body.trim().is_empty() || template.body.chars().count() > BODY_LIMIT {
            problems.push(FieldError::new("body", "length"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("notify.template_invalid", problems));
        }
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, type_code, channel, locale, subject, body) values ($1, $2, $3, $4, $5, $6, $7)
             on conflict (campus_id, type_code, channel, locale) do update set subject = excluded.subject,
                 body = excluded.body, updated_at = now()",
            self.table("template")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(campus.as_uuid())
        .bind(&template.type_code)
        .bind(&template.channel)
        .bind(&locale)
        .bind(template.subject.trim())
        .bind(&template.body)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(Template { locale, ..template })
    }

    /// The templates of a campus.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `notify.template.manage`.
    pub async fn templates(&self, actor: &Actor, campus: ScopeId) -> Result<Vec<Template>> {
        self.require_in(actor, TEMPLATE_MANAGE, campus).await?;
        let rows = sqlx::query(sql(format!(
            "select type_code, channel, locale, subject, body from {} where campus_id = $1
             order by type_code, channel, locale",
            self.table("template")
        )))
        .bind(campus.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Template {
                type_code: r.get("type_code"),
                channel: r.get("channel"),
                locale: r.get("locale"),
                subject: r.get("subject"),
                body: r.get("body"),
            })
            .collect())
    }

    /// Removes a template of a campus; the summary of the type is sent again.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `notify.template.manage`.
    pub async fn remove_template(
        &self,
        actor: &Actor,
        campus: ScopeId,
        type_code: &str,
        channel: &str,
        locale: &str,
    ) -> Result<()> {
        self.require_in(actor, TEMPLATE_MANAGE, campus).await?;
        sqlx::query(sql(format!(
            "delete from {} where campus_id = $1 and type_code = $2 and channel = $3 and locale = $4",
            self.table("template")
        )))
        .bind(campus.as_uuid())
        .bind(type_code)
        .bind(channel)
        .bind(locale.to_lowercase())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    }

    /// Sets what a campus gives by default for a type on a channel, and whether people may change it; needs
    /// `notify.defaults.manage` in the campus.
    ///
    /// # Errors
    ///
    /// `permission_denied`, `not_found` (`notify.type_unknown`), `validation` for an unknown channel.
    pub async fn set_campus_default(
        &self,
        actor: &Actor,
        campus: ScopeId,
        default: CampusDefault,
    ) -> Result<CampusDefault> {
        self.require_in(actor, DEFAULTS_MANAGE, campus).await?;
        self.require_type(&default.type_code).await?;
        if !CHANNELS.contains(&default.channel.as_str()) {
            return Err(Error::validation(
                "notify.default_invalid",
                vec![FieldError::new("channel", "one_of")],
            ));
        }
        sqlx::query(sql(format!(
            "insert into {} (campus_id, type_code, channel, enabled, locked) values ($1, $2, $3, $4, $5)
             on conflict (campus_id, type_code, channel) do update set enabled = excluded.enabled,
                 locked = excluded.locked",
            self.table("campus_default")
        )))
        .bind(campus.as_uuid())
        .bind(&default.type_code)
        .bind(&default.channel)
        .bind(default.enabled)
        .bind(default.locked)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(default)
    }

    /// The defaults a campus sets.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `notify.defaults.manage`.
    pub async fn campus_defaults(&self, actor: &Actor, campus: ScopeId) -> Result<Vec<CampusDefault>> {
        self.require_in(actor, DEFAULTS_MANAGE, campus).await?;
        let rows = sqlx::query(sql(format!(
            "select type_code, channel, enabled, locked from {} where campus_id = $1 order by type_code, channel",
            self.table("campus_default")
        )))
        .bind(campus.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| CampusDefault {
                type_code: r.get("type_code"),
                channel: r.get("channel"),
                enabled: r.get("enabled"),
                locked: r.get("locked"),
            })
            .collect())
    }

    /// Registers a browser or phone of the actor for push notifications; an endpoint registered before,
    /// even by someone else, now belongs to the actor.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor, `validation` (`notify.push_invalid`) for an endpoint that is not
    /// https or missing keys.
    pub async fn register_push(
        &self,
        actor: &Actor,
        subscription: PushSubscription,
    ) -> Result<PushSubscription> {
        let me = person(actor)?;
        let endpoint = subscription.endpoint.trim();
        if !endpoint.starts_with("https://")
            || endpoint.len() > 2000
            || subscription.p256dh.trim().is_empty()
            || subscription.auth.trim().is_empty()
        {
            return Err(Error::validation(
                "notify.push_invalid",
                vec![FieldError::new("endpoint", "format")],
            ));
        }
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {} (id, account_id, endpoint, p256dh, auth) values ($1, $2, $3, $4, $5)
             on conflict (endpoint) do update set account_id = excluded.account_id, p256dh = excluded.p256dh,
                 auth = excluded.auth returning id",
            self.table("push_subscription")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(me.as_uuid())
        .bind(endpoint)
        .bind(subscription.p256dh.trim())
        .bind(subscription.auth.trim())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(PushSubscription {
            id: Some(id),
            endpoint: endpoint.to_owned(),
            ..subscription
        })
    }

    /// The public key browsers subscribe with (`applicationServerKey`).
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.push_unavailable`) when the installation sends no push notifications.
    pub fn push_key(&self) -> Result<String> {
        self.push
            .public_key()
            .ok_or_else(|| Error::not_found("notify.push_unavailable"))
    }

    /// The push subscriptions of the actor.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn push_subscriptions(&self, actor: &Actor) -> Result<Vec<PushSubscription>> {
        let me = person(actor)?;
        let rows = sqlx::query(sql(format!(
            "select id, endpoint, p256dh, auth from {} where account_id = $1 order by created_at",
            self.table("push_subscription")
        )))
        .bind(me.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| PushSubscription {
                id: Some(r.get("id")),
                endpoint: r.get("endpoint"),
                p256dh: r.get("p256dh"),
                auth: r.get("auth"),
            })
            .collect())
    }

    /// Removes a push subscription of the actor.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.push_not_found`) for one of someone else.
    pub async fn remove_push(&self, actor: &Actor, id: uuid::Uuid) -> Result<()> {
        let me = person(actor)?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where id = $1 and account_id = $2",
            self.table("push_subscription")
        )))
        .bind(id)
        .bind(me.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed == 0 {
            return Err(Error::not_found("notify.push_not_found"));
        }
        Ok(())
    }
}
