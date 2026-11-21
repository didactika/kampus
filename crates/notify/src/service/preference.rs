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

//! Preferences, by type and channel, with one rule for every notification: a campus default that is locked
//! wins, then the choice of the person, then the campus default, then the channels the type declares. A
//! required type always reaches the inbox; the campus can route it to more channels.

use std::collections::BTreeMap;
use std::sync::Arc;

use kampus_access::{Access, Actor, PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Result, SystemClock};
use serde_json::Value;
use sqlx::Row;

use super::person;
use crate::{CHANNELS, INBOX, Notify, Preference, sql};

/// What a campus sets for a type and a channel: whether it is on, and whether people may change it.
type CampusSetting = (bool, bool);

impl Notify {
    /// Access, for the campus of a person and the permissions of the settings of a campus.
    pub(crate) fn access(&self) -> Access {
        Access::new(self.db.clone(), Arc::new(SystemClock))
    }

    /// The campus of a person, when they have one.
    pub(crate) async fn campus_of(&self, account: PrincipalId) -> Result<Option<ScopeId>> {
        Ok(self.access().principal_campus(account).await.ok())
    }

    /// What a person receives of each type, or of one type, on every channel.
    pub(crate) async fn resolved(&self, account: PrincipalId, only: Option<&str>) -> Result<Vec<Preference>> {
        let campus = self.campus_of(account).await?;
        let types = sqlx::query(sql(format!(
            "select code, summary, channels, required from {} where retired_at is null
                 and ($1::text is null or code = $1) order by code",
            self.table("notification_type")
        )))
        .bind(only)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let chosen: BTreeMap<(String, String), bool> = sqlx::query(sql(format!(
            "select type_code, channel, enabled from {} where account_id = $1",
            self.table("preference")
        )))
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .iter()
        .map(|r| ((r.get("type_code"), r.get("channel")), r.get("enabled")))
        .collect();
        let campus_set: BTreeMap<(String, String), CampusSetting> = sqlx::query(sql(format!(
            "select type_code, channel, enabled, locked from {} where campus_id = $1",
            self.table("campus_default")
        )))
        .bind(campus.map(|c| c.as_uuid()))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .iter()
        .map(|r| {
            (
                (r.get("type_code"), r.get("channel")),
                (r.get("enabled"), r.get("locked")),
            )
        })
        .collect();
        let mut found = Vec::new();
        for row in &types {
            let code: String = row.get("code");
            let declared: Value = row.get("channels");
            let required: bool = row.get("required");
            for channel in CHANNELS {
                let key = (code.clone(), (*channel).to_owned());
                let by_type = declared
                    .as_array()
                    .is_some_and(|c| c.iter().any(|v| v == channel));
                let campus_value = campus_set.get(&key).copied();
                let locked = campus_value.is_some_and(|(_, l)| l) || (required && *channel == INBOX);
                let enabled = if required && *channel == INBOX {
                    true
                } else if let Some((on, true)) = campus_value {
                    on
                } else {
                    chosen
                        .get(&key)
                        .copied()
                        .or(campus_value.map(|(on, _)| on))
                        .unwrap_or(by_type)
                };
                found.push(Preference {
                    type_code: code.clone(),
                    summary: row.get("summary"),
                    channel: (*channel).to_owned(),
                    enabled,
                    required,
                    locked,
                });
            }
        }
        Ok(found)
    }

    /// What the actor receives of every type, on every channel.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn preferences(&self, actor: &Actor) -> Result<Vec<Preference>> {
        self.resolved(person(actor)?, None).await
    }

    /// Turns a type on or off for the actor on a channel.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.type_unknown`), `validation` (`notify.preference_invalid`) for an unknown channel
    /// or the inbox of a required type, `conflict` (`notify.preference_locked`) for what the campus fixed.
    pub async fn set_preference(
        &self,
        actor: &Actor,
        type_code: &str,
        channel: &str,
        enabled: bool,
    ) -> Result<Preference> {
        let me = person(actor)?;
        if !CHANNELS.contains(&channel) {
            return Err(Error::validation(
                "notify.preference_invalid",
                vec![FieldError::new("channel", "one_of")],
            ));
        }
        let current = self
            .resolved(me, Some(type_code))
            .await?
            .into_iter()
            .find(|p| p.channel == channel)
            .ok_or_else(|| Error::not_found("notify.type_unknown"))?;
        if current.required && channel == INBOX && !enabled {
            return Err(Error::validation(
                "notify.preference_invalid",
                vec![FieldError::new("enabled", "allowed")],
            ));
        }
        if current.locked && current.enabled != enabled {
            return Err(Error::conflict("notify.preference_locked"));
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
        self.resolved(me, Some(type_code))
            .await?
            .into_iter()
            .find(|p| p.channel == channel)
            .ok_or_else(|| Error::not_found("notify.type_unknown"))
    }
}
