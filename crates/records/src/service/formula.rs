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

//! Formulas: a manual record or a total can compute its grades from other records of the course. What a
//! formula reads is kept as relations, a formula that would read itself through others is refused, and the
//! recalculation job computes it with the totals.

use std::collections::{BTreeMap, BTreeSet};

use kampus_access::Actor;
use kampus_common::{Error, FieldError, Result};
use serde::Serialize;
use sqlx::{PgConnection, Row};

use super::principal;
use crate::formula::{Expr, parse};
use crate::permissions::MANAGE;
use crate::{Record, RecordId, RecordKind, Records, events, sql};

/// The formula of a record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Formula {
    /// The record it computes.
    pub record: RecordId,
    /// The expression, with the records it reads written `[[<record id>]]`.
    pub expression: String,
    /// The records it reads.
    pub depends_on: Vec<RecordId>,
}

/// What each derived record reads: its formula, or the children of the category it totals.
pub(crate) fn reads(
    records: &[Record],
    formulas: &BTreeMap<RecordId, Expr>,
    id: RecordId,
) -> BTreeSet<RecordId> {
    if let Some(expr) = formulas.get(&id) {
        return expr.references().into_iter().map(RecordId::from_uuid).collect();
    }
    let Some(category) = records.iter().find(|r| r.id == id).and_then(|r| r.total_of) else {
        return BTreeSet::new();
    };
    records
        .iter()
        .filter(|r| r.category == Some(category))
        .map(|r| r.id)
        .collect()
}

/// The derived records in an order where each comes after those it reads.
pub(crate) fn derived_order(records: &[Record], formulas: &BTreeMap<RecordId, Expr>) -> Vec<RecordId> {
    let derived: BTreeSet<RecordId> = records
        .iter()
        .filter(|r| r.total_of.is_some() || formulas.contains_key(&r.id))
        .map(|r| r.id)
        .collect();
    let mut order = Vec::new();
    let mut placed = BTreeSet::new();
    while placed.len() < derived.len() {
        let ready: Vec<RecordId> = derived
            .iter()
            .filter(|id| !placed.contains(*id))
            .filter(|id| {
                reads(records, formulas, **id)
                    .iter()
                    .all(|d| !derived.contains(d) || placed.contains(d))
            })
            .copied()
            .collect();
        if ready.is_empty() {
            order.extend(derived.iter().filter(|id| !placed.contains(*id)));
            break;
        }
        for id in ready {
            placed.insert(id);
            order.push(id);
        }
    }
    order
}

/// Whether `target` is read, directly or through other derived records, by what `from` reads.
fn reaches(
    records: &[Record],
    formulas: &BTreeMap<RecordId, Expr>,
    from: RecordId,
    target: RecordId,
) -> bool {
    let mut seen = BTreeSet::new();
    let mut pending = vec![from];
    while let Some(id) = pending.pop() {
        if id == target {
            return true;
        }
        if seen.insert(id) {
            pending.extend(reads(records, formulas, id));
        }
    }
    false
}

/// A wrong formula, with what and where.
fn invalid(code: &str, rule: &str) -> Error {
    Error::validation(code, vec![FieldError::new("expression", rule)])
}

impl Records {
    /// The formulas of a course, parsed.
    pub(crate) async fn formulas_in(
        &self,
        conn: &mut PgConnection,
        course: kampus_courses::CourseId,
    ) -> Result<BTreeMap<RecordId, Expr>> {
        let rows = sqlx::query(sql(format!(
            "select f.record_id, f.expression from {} f join {} r on r.id = f.record_id where r.course_id = $1",
            self.table("formula"),
            self.table("record")
        )))
        .bind(course.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .filter_map(|r| {
                let expression: String = r.get("expression");
                parse(&expression)
                    .ok()
                    .map(|e| (RecordId::from_uuid(r.get("record_id")), e))
            })
            .collect())
    }

    /// Whether a record computes its grades with a formula.
    pub(crate) async fn has_formula_in(&self, conn: &mut PgConnection, record: RecordId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where record_id = $1)",
            self.table("formula")
        )))
        .bind(record.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)
    }

    /// The formula of a record, for whoever manages the records of its course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn formula(&self, actor: &Actor, record: RecordId) -> Result<Option<Formula>> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        let expression: Option<String> = sqlx::query_scalar(sql(format!(
            "select expression from {} where record_id = $1",
            self.table("formula")
        )))
        .bind(record.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(expression.map(|expression| {
            let depends_on = parse(&expression).map_or_else(
                |_| Vec::new(),
                |e| e.references().into_iter().map(RecordId::from_uuid).collect(),
            );
            Formula {
                record,
                expression,
                depends_on,
            }
        }))
    }

    /// Gives a manual record or a total a formula, replacing the one it had; needs `records.manage`. The
    /// grades of the course are recalculated.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `conflict` (`records.formula_not_allowed`) for the record of an
    /// activity, `validation` (`records.formula_invalid`) for an expression that does not parse, with the
    /// rule and the position, (`records.formula_reference_unknown`) for a record of another course or none,
    /// and (`records.formula_cycle`) for a formula that would read itself.
    pub async fn set_formula(&self, actor: &Actor, record: RecordId, expression: &str) -> Result<Formula> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        if stored.kind == RecordKind::Activity {
            return Err(Error::conflict("records.formula_not_allowed"));
        }
        let expression = expression.trim();
        let parsed = parse(expression)
            .map_err(|e| invalid("records.formula_invalid", e.rule).with_param("at", e.at.to_string()))?;
        let mut tx = self.db.begin().await?;
        let layout = self.layout_in(&mut tx, course.id).await?;
        let known: BTreeSet<RecordId> = layout.records.iter().map(|r| r.id).collect();
        let reads_now: Vec<RecordId> = parsed.references().into_iter().map(RecordId::from_uuid).collect();
        if let Some(unknown) = reads_now.iter().find(|id| !known.contains(*id)) {
            return Err(invalid("records.formula_reference_unknown", "reference")
                .with_param("record", unknown.to_string()));
        }
        let mut formulas = self.formulas_in(&mut tx, course.id).await?;
        formulas.insert(record, parsed);
        if reads_now
            .iter()
            .any(|d| *d == record || reaches(&layout.records, &formulas, *d, record))
        {
            return Err(invalid("records.formula_cycle", "cycle"));
        }
        sqlx::query(sql(format!(
            "insert into {} (record_id, expression) values ($1, $2)
             on conflict (record_id) do update set expression = excluded.expression, updated_at = now()",
            self.table("formula")
        )))
        .bind(record.as_uuid())
        .bind(expression)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "delete from {} where record_id = $1",
            self.table("formula_dependency")
        )))
        .bind(record.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let ids: Vec<uuid::Uuid> = reads_now.iter().map(RecordId::as_uuid).collect();
        sqlx::query(sql(format!(
            "insert into {} (record_id, depends_on_id) select $1, unnest($2::uuid[])",
            self.table("formula_dependency")
        )))
        .bind(record.as_uuid())
        .bind(&ids)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            events::RECORD_UPDATED,
            &course,
            record,
            None,
            principal(actor),
        )
        .await?;
        self.queue_course_in(&mut tx, course.id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(Formula {
            record,
            expression: expression.to_owned(),
            depends_on: reads_now,
        })
    }

    /// Removes the formula of a record: a manual record keeps the grades it computed, a total goes back to
    /// its category; needs `records.manage`.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn remove_formula(&self, actor: &Actor, record: RecordId) -> Result<()> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        let mut tx = self.db.begin().await?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where record_id = $1",
            self.table("formula")
        )))
        .bind(record.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed > 0 {
            self.publish_in(
                &mut tx,
                events::RECORD_UPDATED,
                &course,
                record,
                None,
                principal(actor),
            )
            .await?;
            self.queue_course_in(&mut tx, course.id).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The formulas that read a record.
    pub(crate) async fn read_by_formulas_in(&self, conn: &mut PgConnection, record: RecordId) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(sql(format!(
            "select count(*) from {} where depends_on_id = $1",
            self.table("formula_dependency")
        )))
        .bind(record.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(u64::try_from(count).unwrap_or(0))
    }
}
