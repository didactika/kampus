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

//! Profiles: names, language and time zone of the person behind an account. A profile exists from the
//! first time it is saved; until then it reads empty. Everyone edits their own; editing someone else's
//! needs `users.profile.update_any` in their campus.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, ErrorKind, FieldError, Result};
use kampus_identity::AccountId;
use kampus_identity::permissions::ACCOUNT_READ;
use sqlx::Row;

use crate::permissions::{PROFILE_UPDATE_ANY, PROFILE_UPDATE_OWN};
use crate::{Profile, ProfileFields, Users, sql};

/// The longest name.
const MAX_NAME: usize = 100;

/// Whether the actor is the account.
pub(crate) fn is_self(actor: &Actor, account: AccountId) -> bool {
    matches!(actor, Actor::Principal(p) if *p == account)
}

/// Checks the fields of a profile, every problem at once, and trims them.
///
/// # Errors
///
/// `validation` (`users.profile_invalid`) naming every field that breaks its rule.
pub fn validate(fields: ProfileFields) -> Result<ProfileFields> {
    let mut problems = Vec::new();
    let clean = |value: Option<String>| value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty());
    let fields = ProfileFields {
        given_name: clean(fields.given_name),
        family_name: clean(fields.family_name),
        display_name: clean(fields.display_name),
        language_code: clean(fields.language_code),
        time_zone: clean(fields.time_zone),
    };
    for (name, value) in [
        ("given_name", &fields.given_name),
        ("family_name", &fields.family_name),
        ("display_name", &fields.display_name),
    ] {
        if value.as_ref().is_some_and(|v| v.chars().count() > MAX_NAME) {
            problems.push(FieldError::new(name, "length"));
        }
    }
    if let Some(code) = &fields.language_code {
        let mut parts = code.split('-');
        let valid = parts
            .next()
            .is_some_and(|p| (2..=3).contains(&p.len()) && p.chars().all(|c| c.is_ascii_lowercase()))
            && parts.all(|p| (2..=8).contains(&p.len()) && p.chars().all(|c| c.is_ascii_alphanumeric()));
        if !valid {
            problems.push(FieldError::new("language_code", "bcp47"));
        }
    }
    if let Some(zone) = &fields.time_zone
        && jiff::tz::TimeZone::get(zone).is_err()
    {
        problems.push(FieldError::new("time_zone", "iana"));
    }
    if problems.is_empty() {
        Ok(fields)
    } else {
        Err(Error::validation("users.profile_invalid", problems))
    }
}

impl Users {
    /// The profile of an account, for the account itself or someone with `identity.account.read` in its
    /// campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown account, `permission_denied` without the permission.
    pub async fn profile(&self, actor: &Actor, account: AccountId) -> Result<Profile> {
        let campus = self.identity.campus_of(account).await?;
        if !is_self(actor, account) {
            self.access.require(actor, ACCOUNT_READ, campus).await?;
        }
        self.read_profile(account).await
    }

    /// Saves the profile of an account: one's own with `users.profile.update_own`, anyone's with
    /// `users.profile.update_any` in their campus. `expected_version` is the version read (`If-Match`).
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields,
    /// `precondition_failed` when the profile changed since it was read.
    pub async fn update_profile(
        &self,
        actor: &Actor,
        account: AccountId,
        fields: ProfileFields,
        expected_version: Option<i32>,
    ) -> Result<Profile> {
        let campus = self.identity.campus_of(account).await?;
        let permission = if is_self(actor, account) {
            PROFILE_UPDATE_OWN
        } else {
            PROFILE_UPDATE_ANY
        };
        self.access.require(actor, permission, campus).await?;
        let fields = validate(fields)?;
        let table = self.table("profile");
        let saved = sqlx::query(sql(format!(
            "insert into {table} as p (account_id, campus_id, given_name, family_name, display_name, language_code, time_zone)
             values ($1, $2, $3, $4, $5, $6, $7)
             on conflict (account_id) do update set given_name = excluded.given_name, family_name = excluded.family_name,
             display_name = excluded.display_name, language_code = excluded.language_code, time_zone = excluded.time_zone,
             version = p.version + 1, updated_at = now()
             where $8::integer is null or p.version = $8
             returning version"
        )))
        .bind(account.as_uuid())
        .bind(campus.as_uuid())
        .bind(&fields.given_name)
        .bind(&fields.family_name)
        .bind(&fields.display_name)
        .bind(&fields.language_code)
        .bind(&fields.time_zone)
        .bind(expected_version)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let Some(row) = saved else {
            let current = self.read_profile(account).await?;
            return Err(Error::new(ErrorKind::PreconditionFailed, "users.profile_changed")
                .with_param("version", current.version.to_string()));
        };
        Ok(Profile {
            given_name: fields.given_name,
            family_name: fields.family_name,
            display_name: fields.display_name,
            language_code: fields.language_code,
            time_zone: fields.time_zone,
            version: row.get("version"),
        })
    }

    /// The names people go by, for a module that lists the participants of a course to those who teach it:
    /// the display name, or the given and family names; those without either are left out.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn names_of(
        &self,
        accounts: &[AccountId],
    ) -> Result<std::collections::BTreeMap<AccountId, String>> {
        let ids: Vec<uuid::Uuid> = accounts.iter().map(AccountId::as_uuid).collect();
        let rows = sqlx::query(sql(format!(
            "select account_id, coalesce(nullif(trim(display_name), ''),
                 nullif(trim(concat_ws(' ', given_name, family_name)), '')) as name
             from {} where account_id = any($1)",
            self.table("profile")
        )))
        .bind(&ids)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .filter_map(|r| {
                r.get::<Option<String>, _>("name")
                    .map(|name| (AccountId::from_uuid(r.get("account_id")), name))
            })
            .collect())
    }

    /// The language a person chose, for a module that writes to them; none when they chose none.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn language_of(&self, account: AccountId) -> Result<Option<String>> {
        Ok(self.read_profile(account).await?.language_code)
    }

    /// The stored profile, or an empty one.
    async fn read_profile(&self, account: AccountId) -> Result<Profile> {
        let row = sqlx::query(sql(format!(
            "select given_name, family_name, display_name, language_code, time_zone, version from {} where account_id = $1",
            self.table("profile")
        )))
        .bind(account.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.map_or_else(Profile::default, |row| Profile {
            given_name: row.get("given_name"),
            family_name: row.get("family_name"),
            display_name: row.get("display_name"),
            language_code: row.get("language_code"),
            time_zone: row.get("time_zone"),
            version: row.get("version"),
        }))
    }

    /// The campus of an account, as identity knows it.
    pub(crate) async fn campus(&self, account: AccountId) -> Result<ScopeId> {
        self.identity.campus_of(account).await
    }
}
