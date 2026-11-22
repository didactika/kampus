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

//! Deliveries: one per notification and channel that is not the inbox, sent by a job after the change that
//! caused it is committed. The text comes from the template of the campus of the person in their language,
//! or from the summary of the type; a failed delivery is tried again later, and given up after its attempts or
//! when the transport refuses it for good.

use kampus_access::PrincipalId;
use kampus_common::{ErrorKind, Result};
use kampus_module::ServiceContext;
use serde_json::Value;
use sqlx::Row;

use crate::ports::transport::Envelope;
use crate::{EMAIL, Notify, sql};

/// How many deliveries one run sends.
const BATCH: i64 = 100;

/// How many times a delivery is tried before it is given up.
pub const MAX_ATTEMPTS: i32 = 5;

/// The language of a template when the person chose none, or none was written in theirs.
pub const DEFAULT_LOCALE: &str = "en";

/// Fills the placeholders of a template (`{{name}}`) with the fields of the payload and the summary of the
/// type; a placeholder without a value is left empty.
#[must_use]
pub fn render(template: &str, summary: &str, payload: &Value) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let key = after[..end].trim();
        let value = if key == "summary" {
            summary.to_owned()
        } else {
            match payload.get(key) {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Null) | None => String::new(),
                Some(other) => other.to_string(),
            }
        };
        out.push_str(&value);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

impl Notify {
    /// The subject and body of a notification for a channel, in the language of the person.
    async fn texts(
        &self,
        account: PrincipalId,
        type_code: &str,
        channel: &str,
        summary: &str,
        payload: &Value,
        services: &ServiceContext,
    ) -> Result<(String, String)> {
        let campus = self.campus_of(account).await?;
        let language = match services.service::<kampus_users::Users>() {
            Ok(users) => {
                users
                    .language_of(kampus_identity::AccountId::from_uuid(account.as_uuid()))
                    .await?
            }
            Err(_) => None,
        };
        let row = sqlx::query(sql(format!(
            "select subject, body from {} where campus_id = $1 and type_code = $2 and channel = $3
                 and locale in ($4, $5) order by (locale = $4) desc limit 1",
            self.table("template")
        )))
        .bind(campus.map(|c| c.as_uuid()))
        .bind(type_code)
        .bind(channel)
        .bind(language.as_deref().unwrap_or(DEFAULT_LOCALE))
        .bind(DEFAULT_LOCALE)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(match row {
            Some(r) => (
                render(r.get("subject"), summary, payload),
                render(r.get("body"), summary, payload),
            ),
            None => (summary.to_owned(), summary.to_owned()),
        })
    }

    /// Where a notification goes on a channel: the email of the person, or the endpoints of their devices.
    async fn addresses(
        &self,
        account: PrincipalId,
        channel: &str,
        services: &ServiceContext,
    ) -> Result<Vec<String>> {
        if channel == EMAIL {
            let identity = services.service::<kampus_identity::Identity>()?;
            return Ok(identity
                .email_of(kampus_identity::AccountId::from_uuid(account.as_uuid()))
                .await?
                .into_iter()
                .collect());
        }
        sqlx::query_scalar(sql(format!(
            "select endpoint from {} where account_id = $1 order by created_at",
            self.table("push_subscription")
        )))
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    /// Sends the deliveries whose time came, at most a batch per run, and returns how many it tried. Each one
    /// is taken by one run only.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn deliver(&self, services: &ServiceContext) -> Result<u64> {
        let rows = sqlx::query(sql(format!(
            "update {d} set attempts = attempts + 1, next_attempt_at = now() + interval '10 minutes'
             from (select notification_id, channel from {d} where state = 'pending' and next_attempt_at <= now()
                   order by next_attempt_at limit $1 for update skip locked) due
             where {d}.notification_id = due.notification_id and {d}.channel = due.channel
             returning {d}.notification_id, {d}.channel, {d}.attempts",
            d = self.table("delivery")
        )))
        .bind(BATCH)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        for row in &rows {
            let notification: uuid::Uuid = row.get("notification_id");
            let channel: String = row.get("channel");
            let attempts: i32 = row.get("attempts");
            let outcome = self.send_one(notification, &channel, services).await;
            let (state, error, retry_minutes) = match &outcome {
                Ok(()) => ("sent", None, 0),
                Err(e)
                    if e.code() == "notify.no_address"
                        || e.kind() == ErrorKind::Validation
                        || attempts >= MAX_ATTEMPTS =>
                {
                    ("dead", Some(e.code().to_owned()), 0)
                }
                Err(e) => (
                    "pending",
                    Some(e.code().to_owned()),
                    5 * 2_i32.pow(u32::try_from(attempts).unwrap_or(0)),
                ),
            };
            sqlx::query(sql(format!(
                "update {} set state = $3, last_error = $4,
                     sent_at = case when $3 = 'sent' then now() else sent_at end,
                     next_attempt_at = now() + make_interval(mins => $5)
                 where notification_id = $1 and channel = $2",
                self.table("delivery")
            )))
            .bind(notification)
            .bind(&channel)
            .bind(state)
            .bind(error)
            .bind(retry_minutes)
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(u64::try_from(rows.len()).unwrap_or(0))
    }

    /// Renders and sends one delivery.
    async fn send_one(
        &self,
        notification: uuid::Uuid,
        channel: &str,
        services: &ServiceContext,
    ) -> Result<()> {
        let row = sqlx::query(sql(format!(
            "select n.recipient_id, n.type_code, n.payload, t.summary from {} n join {} t on t.code = n.type_code
             where n.id = $1",
            self.table("notification"),
            self.table("notification_type")
        )))
        .bind(notification)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let account = PrincipalId::from_uuid(row.get("recipient_id"));
        let type_code: String = row.get("type_code");
        let payload: Value = row.get("payload");
        let summary: String = row.get("summary");
        let to = self.addresses(account, channel, services).await?;
        if to.is_empty() {
            return Err(kampus_common::Error::conflict("notify.no_address"));
        }
        let (subject, body) = self
            .texts(account, &type_code, channel, &summary, &payload, services)
            .await?;
        let envelope = Envelope {
            channel: channel.to_owned(),
            account: account.as_uuid(),
            to,
            subject,
            body,
        };
        if channel == EMAIL {
            self.email.send(&envelope).await
        } else {
            self.push.send(&envelope).await
        }
    }
}
