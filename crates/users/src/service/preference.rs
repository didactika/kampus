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

//! Preferences: per-person settings that modules declare with a type and a default. Only declared keys
//! are kept, and each value is checked against its declaration.

use kampus_access::Actor;
use kampus_common::{Error, FieldError, Result};
use kampus_identity::AccountId;
use kampus_identity::permissions::ACCOUNT_READ;
use kampus_module::PreferenceDeclaration;
use serde_json::Value;
use sqlx::Row;

use crate::permissions::{PROFILE_UPDATE_ANY, PROFILE_UPDATE_OWN};
use crate::service::profile::is_self;
use crate::{Users, sql};

impl Users {
    /// Every declared preference of an account, with its value or its default, by key.
    ///
    /// # Errors
    ///
    /// `permission_denied` for someone else's preferences without `identity.account.read`.
    pub async fn preferences(&self, actor: &Actor, account: AccountId) -> Result<Vec<(&'static str, Value)>> {
        let campus = self.campus(account).await?;
        if !is_self(actor, account) {
            self.access.require(actor, ACCOUNT_READ, campus).await?;
        }
        let rows = sqlx::query(sql(format!(
            "select key, value from {} where account_id = $1",
            self.table("preference")
        )))
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(self
            .preferences
            .iter()
            .map(|declaration| {
                let stored = rows
                    .iter()
                    .find(|r| r.get::<String, _>("key") == declaration.key)
                    .map(|r| r.get::<Value, _>("value"))
                    .filter(|v| declaration.kind.accepts(v));
                (
                    declaration.key,
                    stored.unwrap_or_else(|| declaration.kind.default_value()),
                )
            })
            .collect())
    }

    /// Sets a preference of an account, or goes back to its default with `None`.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `users.profile.update_own` (or `update_any` for someone else),
    /// `validation` for an undeclared key or a value its declaration refuses.
    pub async fn set_preference(
        &self,
        actor: &Actor,
        account: AccountId,
        key: &str,
        value: Option<Value>,
    ) -> Result<()> {
        let campus = self.campus(account).await?;
        let permission = if is_self(actor, account) {
            PROFILE_UPDATE_OWN
        } else {
            PROFILE_UPDATE_ANY
        };
        self.access.require(actor, permission, campus).await?;
        let Some(declaration) = self.declaration(key) else {
            return Err(Error::validation(
                "users.preference_unknown",
                vec![FieldError::new("key", "declared")],
            ));
        };
        let table = self.table("preference");
        match value {
            None => sqlx::query(sql(format!(
                "delete from {table} where account_id = $1 and key = $2"
            )))
            .bind(account.as_uuid())
            .bind(key)
            .execute(self.db.pool())
            .await
            .map(drop)
            .map_err(kampus_db::classify),
            Some(value) if declaration.kind.accepts(&value) => sqlx::query(sql(format!(
                "insert into {table} (account_id, key, value) values ($1, $2, $3)
                 on conflict (account_id, key) do update set value = excluded.value, updated_at = now()"
            )))
            .bind(account.as_uuid())
            .bind(key)
            .bind(value)
            .execute(self.db.pool())
            .await
            .map(drop)
            .map_err(kampus_db::classify),
            Some(_) => Err(Error::validation(
                "users.preference_invalid",
                vec![FieldError::new("value", "declared_type")],
            )
            .with_param("key", key)),
        }
    }

    /// The declaration of a key.
    fn declaration(&self, key: &str) -> Option<PreferenceDeclaration> {
        self.preferences.iter().find(|d| d.key == key).copied()
    }
}
