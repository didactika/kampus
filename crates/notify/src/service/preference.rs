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

//! Preferences: a person chooses, for each type and channel, whether they receive it; a required type cannot
//! be turned off.

use kampus_access::Actor;
use kampus_common::{Error, FieldError, Result};
use serde_json::Value;
use sqlx::Row;

use super::person;
use crate::{INBOX, Notify, Preference, sql};

impl Notify {
    /// What the actor receives of each declared type through the inbox: their choice, or the default of the
    /// type.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn preferences(&self, actor: &Actor) -> Result<Vec<Preference>> {
        let me = person(actor)?;
        let rows = sqlx::query(sql(format!(
            "select t.code, t.summary, t.channels, t.required, p.enabled from {} t
             left join {} p on p.type_code = t.code and p.account_id = $1 and p.channel = $2
             where t.retired_at is null order by t.code",
            self.table("notification_type"),
            self.table("preference")
        )))
        .bind(me.as_uuid())
        .bind(INBOX)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| {
                let channels: Value = r.get("channels");
                let by_default = channels.as_array().is_some_and(|c| c.iter().any(|v| v == INBOX));
                let required: bool = r.get("required");
                Preference {
                    type_code: r.get("code"),
                    summary: r.get("summary"),
                    channel: INBOX.to_owned(),
                    enabled: required || r.get::<Option<bool>, _>("enabled").unwrap_or(by_default),
                    required,
                }
            })
            .collect())
    }

    /// Turns a type on or off for the actor through a channel.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.type_unknown`) for a type that is not declared, `validation`
    /// (`notify.preference_invalid`) for a channel that does not exist or for turning off a required type.
    pub async fn set_preference(
        &self,
        actor: &Actor,
        type_code: &str,
        channel: &str,
        enabled: bool,
    ) -> Result<Preference> {
        let me = person(actor)?;
        let required: Option<bool> = sqlx::query_scalar(sql(format!(
            "select required from {} where code = $1 and retired_at is null",
            self.table("notification_type")
        )))
        .bind(type_code)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let required = required.ok_or_else(|| Error::not_found("notify.type_unknown"))?;
        if channel != INBOX || (required && !enabled) {
            return Err(Error::validation(
                "notify.preference_invalid",
                vec![FieldError::new(
                    if channel == INBOX { "enabled" } else { "channel" },
                    "allowed",
                )],
            ));
        }
        sqlx::query(sql(format!(
            "insert into {} (account_id, type_code, channel, enabled) values ($1, $2, $3, $4)
             on conflict (account_id, type_code, channel) do update set enabled = excluded.enabled",
            self.table("preference")
        )))
        .bind(me.as_uuid())
        .bind(type_code)
        .bind(channel)
        .bind(enabled)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        self.preferences(actor)
            .await?
            .into_iter()
            .find(|p| p.type_code == type_code)
            .ok_or_else(|| Error::not_found("notify.type_unknown"))
    }
}
