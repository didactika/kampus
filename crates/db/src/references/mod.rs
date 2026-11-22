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
// @package   kampus-db
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! External references: the identifiers other systems give a record, such as the `idnumber` of another
//! platform or the code of a student information system. A reference is (system, external id), unique per
//! campus, system and kind of record, kept in a table of the module that owns the record, never in a
//! column of the record itself, and distinct from the code people read.
//!
//! Each module declares one table per kind of record with [`reference_table!`] and reaches it through
//! [`References`].

use kampus_common::{Error, FieldError, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

use crate::Database;

/// The longest system code, in characters.
const MAX_SYSTEM: usize = 63;

/// The longest external id, in characters.
const MAX_EXTERNAL_ID: usize = 255;

/// The identifier a system gives a record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalReference {
    /// The system, such as `sis` or `lms`: lowercase letters, digits, `_`, `-` and `.`.
    pub system: String,
    /// The identifier the system gives the record.
    pub external_id: String,
}

impl ExternalReference {
    /// A reference that keeps to the rules, trimmed.
    ///
    /// # Errors
    ///
    /// `validation` (`references.invalid`) for a system that is not a code or an empty, long or
    /// multi-line external id.
    pub fn new(system: &str, external_id: &str) -> Result<Self> {
        let (system, external_id) = (system.trim(), external_id.trim());
        let mut problems = Vec::new();
        let code = system.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && system.chars().count() <= MAX_SYSTEM
            && system
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-.".contains(&b));
        if !code {
            problems.push(FieldError::new("system", "code"));
        }
        if external_id.is_empty()
            || external_id.chars().count() > MAX_EXTERNAL_ID
            || external_id.chars().any(char::is_control)
        {
            problems.push(FieldError::new("external_id", "length"));
        }
        if problems.is_empty() {
            Ok(Self {
                system: system.to_owned(),
                external_id: external_id.to_owned(),
            })
        } else {
            Err(Error::validation("references.invalid", problems))
        }
    }
}

/// The external references of one kind of record of a module.
#[derive(Debug, Clone)]
pub struct References {
    /// The database.
    db: Database,
    /// The quoted, prefixed name of the table.
    table: String,
}

impl References {
    /// The references kept in a table of a module.
    #[must_use]
    pub fn new(db: &Database, module: &str, table: &str) -> Self {
        Self {
            db: db.clone(),
            table: format!("{}.\"{table}\"", db.prefix().schema(module)),
        }
    }

    /// Gives a record the identifier a system uses for it, replacing the one that system gave it before.
    ///
    /// # Errors
    ///
    /// `conflict` (`references.taken`) when another record of the kind has that identifier of the system
    /// in the campus.
    pub async fn set_in(
        &self,
        conn: &mut PgConnection,
        campus: uuid::Uuid,
        record: uuid::Uuid,
        reference: &ExternalReference,
    ) -> Result<()> {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into {} (campus_id, system, identifier, record_id) values ($1, $2, $3, $4)
             on conflict (record_id, system) do update set identifier = excluded.identifier",
            self.table
        )))
        .bind(campus)
        .bind(&reference.system)
        .bind(&reference.external_id)
        .bind(record)
        .execute(&mut *conn)
        .await
        .map_err(crate::classify)
        .map_err(|e| {
            if e.code() == "db.unique_violation" {
                Error::conflict("references.taken")
                    .with_param("system", reference.system.clone())
                    .with_param("external_id", reference.external_id.clone())
            } else {
                e
            }
        })?;
        Ok(())
    }

    /// Gives a record the identifier a system uses for it, in a transaction of its own.
    ///
    /// # Errors
    ///
    /// Those of [`References::set_in`].
    pub async fn set(
        &self,
        campus: uuid::Uuid,
        record: uuid::Uuid,
        reference: &ExternalReference,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        self.set_in(&mut tx, campus, record, reference).await?;
        tx.commit().await.map_err(crate::classify)
    }

    /// Removes the identifier a system gave a record; whether there was one.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn remove(&self, record: uuid::Uuid, system: &str) -> Result<bool> {
        let removed = sqlx::query(sqlx::AssertSqlSafe(format!(
            "delete from {} where record_id = $1 and system = $2",
            self.table
        )))
        .bind(record)
        .bind(system)
        .execute(self.db.pool())
        .await
        .map_err(crate::classify)?
        .rows_affected();
        Ok(removed > 0)
    }

    /// The record of the kind a system knows by an identifier in a campus.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn find(
        &self,
        campus: uuid::Uuid,
        reference: &ExternalReference,
    ) -> Result<Option<uuid::Uuid>> {
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "select record_id from {} where campus_id = $1 and system = $2 and identifier = $3",
            self.table
        )))
        .bind(campus)
        .bind(&reference.system)
        .bind(&reference.external_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(crate::classify)
    }

    /// The identifiers systems give a record, by system.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn of(&self, record: uuid::Uuid) -> Result<Vec<ExternalReference>> {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "select system, identifier from {} where record_id = $1 order by system",
            self.table
        )))
        .bind(record)
        .fetch_all(self.db.pool())
        .await
        .map_err(crate::classify)?;
        Ok(rows
            .iter()
            .map(|r| ExternalReference {
                system: r.get("system"),
                external_id: r.get("identifier"),
            })
            .collect())
    }
}

/// The table of the external references of one kind of record: `reference_table!("course_reference",
/// "course")` keeps the references of `course` rows of the same module, which take their references with
/// them when deleted.
#[macro_export]
macro_rules! reference_table {
    ($name:literal, $record:literal) => {
        $crate::schema::Change::CreateTable($crate::schema::Table {
            name: $name,
            columns: &[
                $crate::schema::Column::new("campus_id", $crate::schema::Type::Uuid),
                $crate::schema::Column::new("system", $crate::schema::Type::Text),
                $crate::schema::Column::new("identifier", $crate::schema::Type::Text),
                $crate::schema::Column::new("record_id", $crate::schema::Type::Uuid),
                $crate::schema::Column::new("created_at", $crate::schema::Type::Timestamptz)
                    .default($crate::schema::ColumnDefault::Now),
            ],
            primary_key: &["campus_id", "system", "identifier"],
            unique: &[&["record_id", "system"]],
            foreign_keys: &[
                $crate::schema::ForeignKey {
                    columns: &["campus_id"],
                    module: Some("access"),
                    table: "scope",
                    references: &["id"],
                    on_delete: $crate::schema::OnDelete::Cascade,
                },
                $crate::schema::ForeignKey {
                    columns: &["record_id"],
                    module: None,
                    table: $record,
                    references: &["id"],
                    on_delete: $crate::schema::OnDelete::Cascade,
                },
            ],
            without_natural_key: Some("keyed by the identifier the system gives the record in the campus"),
            ..$crate::schema::Table::EMPTY
        })
    };
}
