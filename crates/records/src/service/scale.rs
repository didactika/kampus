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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Scales and letter sets: lists of items and bands of percentages defined in a scope and available there
//! and below. A new version is a new row; a record keeps the version it was set up with.

use std::collections::HashSet;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use kampus_courses::{Course, CourseId};
use rust_decimal::Decimal;
use sqlx::{PgConnection, Row};

use super::problem;
use crate::permissions::{MANAGE, VIEW_OWN};
use crate::support::letters::letter_for;
use crate::{Band, GradeType, LetterSet, LetterSetId, RecordFields, Records, Scale, ScaleId, sql};

/// The most items of a scale.
const ITEMS_LIMIT: usize = 50;

/// The most characters of an item or a letter.
const TEXT_LIMIT: usize = 60;

/// A scale from its row.
fn scale(row: &sqlx::postgres::PgRow) -> Scale {
    Scale {
        id: ScaleId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        code: row.get("code"),
        version: row.get("version"),
        items: row.get("items"),
    }
}

impl Records {
    /// The scopes from the one given up to the root, nearest first.
    async fn chain_in(&self, conn: &mut PgConnection, scope: ScopeId) -> Result<Vec<ScopeId>> {
        Ok(self
            .access
            .path(conn, scope)
            .await?
            .into_iter()
            .map(|p| p.id)
            .collect())
    }

    /// Fails unless the actor may look at what is defined in a scope: whoever sees records there or
    /// configures them.
    async fn require_definitions(&self, actor: &Actor, scope: ScopeId) -> Result<()> {
        if self.access.check(actor, VIEW_OWN, scope).await?.allowed {
            return Ok(());
        }
        self.access.require(actor, MANAGE, scope).await
    }

    /// Defines a scale in a scope, as the next version when the code exists there; needs `records.manage`
    /// in the scope.
    ///
    /// # Errors
    ///
    /// `permission_denied`, `validation` (`records.scale_invalid`): a code of lowercase letters, digits and
    /// underscores, and two to fifty different items.
    pub async fn define_scale(
        &self,
        actor: &Actor,
        scope: ScopeId,
        code: &str,
        items: Vec<String>,
    ) -> Result<Scale> {
        self.access.require(actor, MANAGE, scope).await?;
        let items: Vec<String> = items.iter().map(|i| i.trim().to_owned()).collect();
        let mut problems = Vec::new();
        if code.is_empty()
            || code.len() > 40
            || !code
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            problems.push(problem("code", "format"));
        }
        let different: HashSet<&String> = items.iter().collect();
        if items.len() < 2 || items.len() > ITEMS_LIMIT {
            problems.push(problem("items", "count"));
        }
        if different.len() != items.len()
            || items
                .iter()
                .any(|i| i.is_empty() || i.chars().count() > TEXT_LIMIT)
        {
            problems.push(problem("items", "different_and_short"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("records.scale_invalid", problems));
        }
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {t} (id, scope_id, code, version, items)
             select $1, $2, $3, coalesce(max(version), 0) + 1, $4 from {t} where scope_id = $2 and code = $3
             returning id, scope_id, code, version, items",
            t = self.table("scale")
        )))
        .bind(ScaleId::generate().as_uuid())
        .bind(scope.as_uuid())
        .bind(code)
        .bind(&items)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(scale(&row))
    }

    /// The scales available in a scope: the latest version of each code defined there or above, the nearest
    /// definition winning.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope, `permission_denied`.
    pub async fn scales_in(&self, actor: &Actor, scope: ScopeId) -> Result<Vec<Scale>> {
        self.require_definitions(actor, scope).await?;
        self.available_scales(scope).await
    }

    /// The scales available in a scope, whoever asks.
    async fn available_scales(&self, scope: ScopeId) -> Result<Vec<Scale>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let chain = self.chain_in(&mut conn, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select id, scope_id, code, version, items from {} where scope_id = any($1)
             order by version desc, code",
            self.table("scale")
        )))
        .bind(chain.iter().map(ScopeId::as_uuid).collect::<Vec<_>>())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let all: Vec<Scale> = rows.iter().map(scale).collect();
        let mut seen = HashSet::new();
        let mut found = Vec::new();
        for at in &chain {
            for candidate in all.iter().filter(|s| s.scope == *at) {
                if seen.insert(candidate.code.clone()) {
                    found.push(candidate.clone());
                }
            }
        }
        found.sort_by(|a, b| a.code.cmp(&b.code));
        Ok(found)
    }

    /// The scales available in a course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn scales_of_course(&self, actor: &Actor, course: CourseId) -> Result<Vec<Scale>> {
        let course = self.courses.course(course).await?;
        self.require_view(actor, &course).await?;
        self.available_scales(course.scope).await
    }

    /// The version of a scale a course may use: it is defined in the course or above.
    async fn scale_for_course(&self, course: &Course, id: ScaleId) -> Result<Scale> {
        let row = sqlx::query(sql(format!(
            "select id, scope_id, code, version, items from {} where id = $1",
            self.table("scale")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("records.scale_not_found"))?;
        let found = scale(&row);
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        if self
            .chain_in(&mut conn, course.scope)
            .await?
            .contains(&found.scope)
        {
            Ok(found)
        } else {
            Err(Error::not_found("records.scale_not_found"))
        }
    }

    /// The fields of a record with what its type decides: a scale record takes the range of its scale,
    /// from the first item to the last, and only a scale record has a scale.
    pub(crate) async fn prepared(&self, course: &Course, mut fields: RecordFields) -> Result<RecordFields> {
        match (fields.grade_type, fields.scale) {
            (GradeType::Scale, Some(id)) => {
                let found = self.scale_for_course(course, id).await?;
                fields.min = Decimal::ONE;
                fields.max = Decimal::from(found.items.len());
            }
            (GradeType::Scale, None) => {
                return Err(Error::validation(
                    "records.record_invalid",
                    vec![problem("scale", "required")],
                ));
            }
            (_, Some(_)) => {
                return Err(Error::validation(
                    "records.record_invalid",
                    vec![problem("scale", "unused")],
                ));
            }
            (_, None) => {}
        }
        Ok(fields)
    }

    /// Defines the letter set of a scope as its next version; needs `records.manage` in the scope.
    ///
    /// # Errors
    ///
    /// `permission_denied`, `validation` (`records.letters_invalid`): one to twenty bands from 0 to 100, one
    /// of them starting at 0, with different percentages and short letters.
    pub async fn define_letters(&self, actor: &Actor, scope: ScopeId, bands: Vec<Band>) -> Result<LetterSet> {
        self.access.require(actor, MANAGE, scope).await?;
        let mut problems = Vec::new();
        let percentages: HashSet<Decimal> = bands.iter().map(|b| b.min_percent).collect();
        if bands.is_empty() || bands.len() > 20 || percentages.len() != bands.len() {
            problems.push(problem("bands", "count_and_different"));
        }
        if bands
            .iter()
            .any(|b| b.min_percent < Decimal::ZERO || b.min_percent > Decimal::ONE_HUNDRED)
        {
            problems.push(problem("min_percent", "range"));
        }
        if !bands.iter().any(|b| b.min_percent == Decimal::ZERO) {
            problems.push(problem("bands", "from_zero"));
        }
        if bands
            .iter()
            .any(|b| b.letter.trim().is_empty() || b.letter.chars().count() > TEXT_LIMIT)
        {
            problems.push(problem("letter", "length"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("records.letters_invalid", problems));
        }
        let mut tx = self.db.begin().await?;
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {t} (id, scope_id, version)
             select $1, $2, coalesce(max(version), 0) + 1 from {t} where scope_id = $2 returning id",
            t = self.table("letter_set")
        )))
        .bind(LetterSetId::generate().as_uuid())
        .bind(scope.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for band in &bands {
            sqlx::query(sql(format!(
                "insert into {} (letter_set_id, min_percent, letter) values ($1, $2, $3)",
                self.table("letter_band")
            )))
            .bind(id)
            .bind(band.min_percent)
            .bind(band.letter.trim())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.letter_set_by_id(LetterSetId::from_uuid(id)).await
    }

    /// A letter set with its bands, from the highest percentage down.
    async fn letter_set_by_id(&self, id: LetterSetId) -> Result<LetterSet> {
        let row = sqlx::query(sql(format!(
            "select id, scope_id, version from {} where id = $1",
            self.table("letter_set")
        )))
        .bind(id.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let bands = sqlx::query(sql(format!(
            "select min_percent, letter from {} where letter_set_id = $1 order by min_percent desc",
            self.table("letter_band")
        )))
        .bind(id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(LetterSet {
            id,
            scope: ScopeId::from_uuid(row.get("scope_id")),
            version: row.get("version"),
            bands: bands
                .iter()
                .map(|b| Band {
                    min_percent: b.get::<Decimal, _>("min_percent").normalize(),
                    letter: b.get("letter"),
                })
                .collect(),
        })
    }

    /// The letter set that applies in a scope: the latest version of the nearest scope that has one.
    async fn applicable_letters(&self, scope: ScopeId) -> Result<Option<LetterSet>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        for at in self.chain_in(&mut conn, scope).await? {
            let found: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
                "select id from {} where scope_id = $1 order by version desc limit 1",
                self.table("letter_set")
            )))
            .bind(at.as_uuid())
            .fetch_optional(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
            if let Some(id) = found {
                return Ok(Some(self.letter_set_by_id(LetterSetId::from_uuid(id)).await?));
            }
        }
        Ok(None)
    }

    /// The letter set that applies in a scope.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope, `permission_denied`.
    pub async fn letters_in(&self, actor: &Actor, scope: ScopeId) -> Result<Option<LetterSet>> {
        self.require_definitions(actor, scope).await?;
        self.applicable_letters(scope).await
    }

    /// The letter set that applies in a course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn letters_of_course(&self, actor: &Actor, course: CourseId) -> Result<Option<LetterSet>> {
        let course = self.courses.course(course).await?;
        self.require_view(actor, &course).await?;
        self.applicable_letters(course.scope).await
    }

    /// The letter of a percentage in a course, with the set that applies there; none when no scope has a set
    /// or the percentage is below every band. It is meant for the screens and reports that show grades.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course.
    pub async fn letter_for_course(&self, course: CourseId, percent: Decimal) -> Result<Option<String>> {
        let course = self.courses.course(course).await?;
        Ok(self
            .applicable_letters(course.scope)
            .await?
            .and_then(|set| letter_for(&set.bands, percent).map(str::to_owned)))
    }
}
