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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Quotas: how much the attached files of a scope and of the scopes inside it, and of one person, may
//! take. Every file counts in each scope of its path and in its uploader's quota from the moment it is
//! attached until it is detached, in the same transaction; an upload that would not fit is refused when it
//! is opened, and an attachment that no longer fits is refused inside its transaction.

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde::Serialize;
use sqlx::{PgConnection, Row};

use crate::permissions::QUOTA_MANAGE;
use crate::{Files, sql};

/// How much files may take and take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Quota {
    /// The most, in bytes; no limit when absent.
    pub limit_bytes: Option<i64>,
    /// What attached files take, in bytes.
    pub used_bytes: i64,
}

/// The error of a limit a file does not fit.
fn exceeded(what: &str, limit: i64) -> Error {
    Error::conflict("files.quota_exceeded")
        .with_param("quota", what.to_owned())
        .with_param("limit_bytes", limit.to_string())
}

impl Files {
    /// The scopes a file of a scope counts in: the scope and every scope above it.
    async fn counted_scopes(&self, conn: &mut PgConnection, scope: ScopeId) -> Result<Vec<uuid::Uuid>> {
        Ok(self
            .access
            .path(conn, scope)
            .await?
            .iter()
            .map(|s| s.id.as_uuid())
            .collect())
    }

    /// Fails unless a file of a size fits every quota it would count in.
    pub(crate) async fn require_room(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        uploader: Option<uuid::Uuid>,
        size: i64,
    ) -> Result<()> {
        let scopes = self.counted_scopes(conn, scope).await?;
        let full: Option<i64> = sqlx::query_scalar(sql(format!(
            "select limit_bytes from {} where scope_id = any($1) and used_bytes + $2 > limit_bytes limit 1",
            self.table("quota")
        )))
        .bind(&scopes)
        .bind(size)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(limit) = full {
            return Err(exceeded("scope", limit));
        }
        let personal: Option<i64> = sqlx::query_scalar(sql(format!(
            "select limit_bytes from {} where principal_id = $1 and used_bytes + $2 > limit_bytes",
            self.table("principal_quota")
        )))
        .bind(uploader)
        .bind(size)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        match personal {
            Some(limit) => Err(exceeded("person", limit)),
            None => Ok(()),
        }
    }

    /// Counts the size of a file in, or out with a negative size, of every quota it counts in; a file that
    /// no longer fits is refused, so the caller's transaction rolls back.
    pub(crate) async fn count_in(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        uploader: Option<uuid::Uuid>,
        size: i64,
    ) -> Result<()> {
        let scopes = self.counted_scopes(conn, scope).await?;
        let over = sqlx::query(sql(format!(
            "insert into {t} (scope_id, used_bytes) select unnest($1::uuid[]), greatest($2, 0)
             on conflict (scope_id) do update set used_bytes = greatest({t}.used_bytes + $2, 0)
             returning limit_bytes, used_bytes",
            t = self.table("quota")
        )))
        .bind(&scopes)
        .bind(size)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .iter()
        .find_map(|r| {
            r.get::<Option<i64>, _>("limit_bytes")
                .filter(|l| r.get::<i64, _>("used_bytes") > *l)
        });
        if let (Some(limit), true) = (over, size > 0) {
            return Err(exceeded("scope", limit));
        }
        let Some(uploader) = uploader else { return Ok(()) };
        let statement = if size > 0 {
            "insert into {t} (principal_id, used_bytes) values ($1, $2)
             on conflict (principal_id) do update set used_bytes = {t}.used_bytes + $2
             returning limit_bytes, used_bytes"
        } else {
            "update {t} set used_bytes = greatest(used_bytes + $2, 0) where principal_id = $1
             returning limit_bytes, used_bytes"
        };
        let row = sqlx::query(sql(statement.replace("{t}", &self.table("principal_quota"))))
            .bind(uploader)
            .bind(size)
            .fetch_optional(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        if let Some(row) = row
            && size > 0
            && let Some(limit) = row.get::<Option<i64>, _>("limit_bytes")
            && row.get::<i64, _>("used_bytes") > limit
        {
            return Err(exceeded("person", limit));
        }
        Ok(())
    }

    /// The quota of a scope; needs `files.quota.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn quota(&self, actor: &Actor, scope: ScopeId) -> Result<Quota> {
        self.access.require(actor, QUOTA_MANAGE, scope).await?;
        self.read_quota("quota", "scope_id", scope.as_uuid()).await
    }

    /// Sets how much the files of a scope may take, or lifts the limit; needs `files.quota.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` (`files.quota_invalid`) for a negative
    /// limit.
    pub async fn set_quota(&self, actor: &Actor, scope: ScopeId, limit: Option<i64>) -> Result<Quota> {
        self.access.require(actor, QUOTA_MANAGE, scope).await?;
        self.write_quota("quota", "scope_id", scope.as_uuid(), limit)
            .await
    }

    /// The quota of a person; needs `files.quota.manage` in their campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown person, `permission_denied` without the permission.
    pub async fn person_quota(&self, actor: &Actor, person: PrincipalId) -> Result<Quota> {
        let campus = self.access.principal_campus(person).await?;
        self.access.require(actor, QUOTA_MANAGE, campus).await?;
        self.read_quota("principal_quota", "principal_id", person.as_uuid())
            .await
    }

    /// Sets how much the files of a person may take, or lifts the limit; needs `files.quota.manage` in
    /// their campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown person, `permission_denied` without the permission, `validation`
    /// (`files.quota_invalid`) for a negative limit.
    pub async fn set_person_quota(
        &self,
        actor: &Actor,
        person: PrincipalId,
        limit: Option<i64>,
    ) -> Result<Quota> {
        let campus = self.access.principal_campus(person).await?;
        self.access.require(actor, QUOTA_MANAGE, campus).await?;
        self.write_quota("principal_quota", "principal_id", person.as_uuid(), limit)
            .await
    }

    /// A quota as stored; no limit and nothing used when there is none.
    async fn read_quota(&self, table: &str, key: &str, id: uuid::Uuid) -> Result<Quota> {
        let row = sqlx::query(sql(format!(
            "select limit_bytes, used_bytes from {} where {key} = $1",
            self.table(table)
        )))
        .bind(id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.map_or(
            Quota {
                limit_bytes: None,
                used_bytes: 0,
            },
            |r| Quota {
                limit_bytes: r.get("limit_bytes"),
                used_bytes: r.get("used_bytes"),
            },
        ))
    }

    /// Stores the limit of a quota.
    async fn write_quota(&self, table: &str, key: &str, id: uuid::Uuid, limit: Option<i64>) -> Result<Quota> {
        if limit.is_some_and(|l| l < 0) {
            return Err(Error::validation(
                "files.quota_invalid",
                vec![FieldError::new("limit_bytes", "min")],
            ));
        }
        let row = sqlx::query(sql(format!(
            "insert into {t} ({key}, limit_bytes) values ($1, $2)
             on conflict ({key}) do update set limit_bytes = excluded.limit_bytes returning limit_bytes, used_bytes",
            t = self.table(table)
        )))
        .bind(id)
        .bind(limit)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(Quota {
            limit_bytes: row.get("limit_bytes"),
            used_bytes: row.get("used_bytes"),
        })
    }
}
