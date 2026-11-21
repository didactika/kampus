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

//! Derived grades: the totals of categories and of the course follow the grades of their children. A change
//! queues the people it touches, a job recomputes them in the order the tree gives, and reading never
//! recalculates. The range of the total of a natural category follows its children when they change.

use std::collections::{BTreeMap, HashMap};

use kampus_access::PrincipalId;
use kampus_common::Result;
use kampus_courses::{Course, CourseId};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgConnection, Row};

use super::formula::derived_order;
use super::{CATEGORY_COLUMNS, RECORD_COLUMNS, category, record};
use crate::aggregation::{Child, Rule, aggregate, natural_range};
use crate::{Category, CategoryId, GradeType, Method, Record, RecordId, Records, events, sql};

/// How many queued people one run takes.
const BATCH: i64 = 200;

/// What a run of recalculation did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Recalculated {
    /// The people recalculated.
    pub people: u64,
    /// The totals that changed.
    pub totals_changed: u64,
}

/// The categories and records of a course.
pub(crate) struct Layout {
    /// The categories.
    pub(crate) categories: Vec<Category>,
    /// The records.
    pub(crate) records: Vec<Record>,
}

/// What a person has on a record, for the aggregation.
#[derive(Debug, Clone, Copy)]
struct Held {
    /// What counts of the grade.
    value: Option<Decimal>,
    /// Whether it is left out of the totals.
    excluded: bool,
    /// Whether a teacher overrode it.
    overridden: bool,
}

/// How deep each category is: the root is zero.
fn depths(categories: &[Category]) -> HashMap<CategoryId, usize> {
    let parents: HashMap<CategoryId, Option<CategoryId>> =
        categories.iter().map(|c| (c.id, c.parent)).collect();
    categories
        .iter()
        .map(|c| {
            let mut depth = 0;
            let mut at = c.parent;
            while let Some(parent) = at {
                depth += 1;
                at = parents.get(&parent).copied().flatten();
            }
            (c.id, depth)
        })
        .collect()
}

/// The categories from the deepest to the root, so that a total is computed after those it contains.
fn bottom_up(categories: &[Category]) -> Vec<&Category> {
    let depth = depths(categories);
    let mut ordered: Vec<&Category> = categories.iter().collect();
    ordered.sort_by_key(|c| std::cmp::Reverse(depth.get(&c.id).copied().unwrap_or(0)));
    ordered
}

/// The records that take part in the total of a category: its children that grade with numbers.
fn children_of(records: &[Record], category: CategoryId) -> impl Iterator<Item = &Record> {
    records
        .iter()
        .filter(move |r| r.category == Some(category) && r.grade_type == GradeType::Value)
}

impl Records {
    /// The categories and records of a course, whoever asks.
    pub(crate) async fn layout_in(&self, conn: &mut PgConnection, course: CourseId) -> Result<Layout> {
        let categories = sqlx::query(sql(format!(
            "select {CATEGORY_COLUMNS} from {} where course_id = $1",
            self.table("category")
        )))
        .bind(course.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let records = sqlx::query(sql(format!(
            "select {RECORD_COLUMNS} from {} where course_id = $1",
            self.table("record")
        )))
        .bind(course.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(Layout {
            categories: categories.iter().map(category).collect(),
            records: records.iter().map(record).collect(),
        })
    }

    /// Gives the total of each natural category the range its children add up to, from the deepest up.
    pub(crate) async fn refresh_ranges_in(&self, conn: &mut PgConnection, course: CourseId) -> Result<()> {
        let mut layout = self.layout_in(&mut *conn, course).await?;
        for category in bottom_up(&layout.categories) {
            if category.method != Method::Natural {
                continue;
            }
            let children: Vec<Child> = children_of(&layout.records, category.id)
                .map(|r| Child {
                    grade: None,
                    min: r.min,
                    max: r.max,
                    weight: r.weight,
                    extra_credit: r.extra_credit,
                })
                .collect();
            let (low, mut high) = natural_range(&children);
            if high <= low {
                high = Decimal::ONE_HUNDRED;
            }
            let Some(total) = layout
                .records
                .iter_mut()
                .find(|r| r.total_of == Some(category.id))
            else {
                continue;
            };
            if total.min == low && total.max == high {
                continue;
            }
            sqlx::query(sql(format!(
                "update {} set \"min\" = $2, \"max\" = $3, version = version + 1 where id = $1",
                self.table("record")
            )))
            .bind(total.id.as_uuid())
            .bind(low)
            .bind(high)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
            total.min = low;
            total.max = high;
        }
        Ok(())
    }

    /// Queues the people for recalculation, and a job to do it when something new was queued.
    pub(crate) async fn queue_in(
        &self,
        conn: &mut PgConnection,
        course: CourseId,
        accounts: &[PrincipalId],
    ) -> Result<()> {
        let people: Vec<uuid::Uuid> = accounts.iter().map(PrincipalId::as_uuid).collect();
        let queued = sqlx::query(sql(format!(
            "insert into {} (course_id, account_id) select $1, unnest($2::uuid[]) on conflict do nothing",
            self.table("recalculation")
        )))
        .bind(course.as_uuid())
        .bind(&people)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        self.wake_in(conn, queued).await
    }

    /// Queues everyone with a grade in the course, after a change to its structure, and brings the ranges of
    /// the natural totals up to date.
    pub(crate) async fn queue_course_in(&self, conn: &mut PgConnection, course: CourseId) -> Result<()> {
        self.refresh_ranges_in(conn, course).await?;
        let queued = sqlx::query(sql(format!(
            "insert into {} (course_id, account_id)
             select distinct r.course_id, g.account_id from {} g join {} r on r.id = g.record_id
             where r.course_id = $1 on conflict do nothing",
            self.table("recalculation"),
            self.table("grade"),
            self.table("record")
        )))
        .bind(course.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        self.wake_in(conn, queued).await
    }

    /// Asks the task runner to recalculate when something was queued.
    async fn wake_in(&self, conn: &mut PgConnection, queued: u64) -> Result<()> {
        if queued > 0 {
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(conn, "records.recalculate", &json!({}), None)
                .await?;
        }
        Ok(())
    }

    /// Recalculates the totals of the people queued, in one transaction, and says what changed. A person
    /// whose course is gone is dropped from the queue.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; the queue keeps what was not done.
    pub async fn recalculate(&self) -> Result<Recalculated> {
        let mut tx = self.db.begin().await?;
        let rows = sqlx::query(sql(format!(
            "delete from {t} q using (select course_id, account_id from {t} order by queued_at
                 limit $1 for update skip locked) s
             where q.course_id = s.course_id and q.account_id = s.account_id returning q.course_id, q.account_id",
            t = self.table("recalculation")
        )))
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut by_course: BTreeMap<uuid::Uuid, Vec<PrincipalId>> = BTreeMap::new();
        for row in &rows {
            by_course
                .entry(row.get("course_id"))
                .or_default()
                .push(PrincipalId::from_uuid(row.get("account_id")));
        }
        let mut done = Recalculated::default();
        for (course, people) in by_course {
            let Ok(course) = self.courses.course(CourseId::from_uuid(course)).await else {
                continue;
            };
            done.totals_changed += self.recalculate_course_in(&mut tx, &course, &people).await?;
            done.people += u64::try_from(people.len()).unwrap_or(u64::MAX);
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(done)
    }

    /// Recalculates the totals of some people in a course; returns how many totals changed.
    async fn recalculate_course_in(
        &self,
        conn: &mut PgConnection,
        course: &Course,
        people: &[PrincipalId],
    ) -> Result<u64> {
        let layout = self.layout_in(&mut *conn, course.id).await?;
        let ids: Vec<uuid::Uuid> = people.iter().map(PrincipalId::as_uuid).collect();
        let rows = sqlx::query(sql(format!(
            "select g.record_id, g.account_id, g.\"final\", g.excluded, g.overridden from {} g
             join {} r on r.id = g.record_id where r.course_id = $1 and g.account_id = any($2)",
            self.table("grade"),
            self.table("record")
        )))
        .bind(course.id.as_uuid())
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut held: HashMap<(PrincipalId, RecordId), Held> = HashMap::new();
        for row in &rows {
            held.insert(
                (
                    PrincipalId::from_uuid(row.get("account_id")),
                    RecordId::from_uuid(row.get("record_id")),
                ),
                Held {
                    value: row.get::<Option<Decimal>, _>("final").map(|d| d.normalize()),
                    excluded: row.get("excluded"),
                    overridden: row.get("overridden"),
                },
            );
        }
        let formulas = self.formulas_in(&mut *conn, course.id).await?;
        let order = derived_order(&layout.records, &formulas);
        let mut changed = 0;
        for person in people {
            for id in &order {
                let Some(total) = layout.records.iter().find(|r| r.id == *id) else {
                    continue;
                };
                let value = if let Some(expr) = formulas.get(id) {
                    expr.evaluate(&|read| {
                        held.get(&(*person, RecordId::from_uuid(read)))
                            .filter(|h| !h.excluded)
                            .and_then(|h| h.value)
                    })
                    .map(|v| v.round_dp(5).max(total.min).min(total.max))
                } else if let Some(category) = total
                    .total_of
                    .and_then(|c| layout.categories.iter().find(|k| k.id == c))
                {
                    category_value(&layout, category, total, &held, *person)
                } else {
                    continue;
                };
                let (counted, did) = self
                    .store_total_in(
                        conn,
                        course,
                        total,
                        *person,
                        value,
                        held.get(&(*person, total.id)).copied(),
                    )
                    .await?;
                changed += u64::from(did);
                held.insert(
                    (*person, total.id),
                    Held {
                        value: counted,
                        excluded: held.get(&(*person, total.id)).is_some_and(|h| h.excluded),
                        overridden: held.get(&(*person, total.id)).is_some_and(|h| h.overridden),
                    },
                );
            }
        }
        Ok(changed)
    }
}

/// The total of a category for a person, from the grades of its children.
fn category_value(
    layout: &Layout,
    category: &Category,
    total: &Record,
    held: &HashMap<(PrincipalId, RecordId), Held>,
    person: PrincipalId,
) -> Option<Decimal> {
    let children: Vec<Child> = children_of(&layout.records, category.id)
        .map(|r| Child {
            grade: held
                .get(&(person, r.id))
                .filter(|h| !h.excluded)
                .and_then(|h| h.value),
            min: r.min,
            max: r.max,
            weight: r.weight,
            extra_credit: r.extra_credit,
        })
        .collect();
    let rule = Rule {
        method: category.method,
        drop_lowest: usize::try_from(category.drop_lowest).unwrap_or(0),
        keep_highest: usize::try_from(category.keep_highest).unwrap_or(0),
        exclude_empty: category.exclude_empty,
    };
    let range = if category.method == Method::Natural {
        natural_range(&children)
    } else {
        (total.min, total.max)
    };
    aggregate(rule, &children, range)
}

impl Records {
    /// Writes the total of a person when it differs from what is stored, and says what counts for the
    /// total above it: an override stays, and a total without grades has no row.
    async fn store_total_in(
        &self,
        conn: &mut PgConnection,
        course: &Course,
        total: &Record,
        person: PrincipalId,
        value: Option<Decimal>,
        existing: Option<Held>,
    ) -> Result<(Option<Decimal>, bool)> {
        let overridden = existing.is_some_and(|h| h.overridden);
        let changed = match value {
            Some(v) => sqlx::query(sql(format!(
                "insert into {t} (record_id, account_id, raw, \"final\") values ($1, $2, $3, $3)
                     on conflict (record_id, account_id) do update set raw = excluded.raw,
                         \"final\" = case when {t}.overridden then {t}.\"final\" else excluded.\"final\" end,
                         version = {t}.version + 1, updated_at = now()
                     where {t}.raw is distinct from excluded.raw
                         or (not {t}.overridden and {t}.\"final\" is distinct from excluded.\"final\")
                     returning 1",
                t = self.table("grade")
            )))
            .bind(total.id.as_uuid())
            .bind(person.as_uuid())
            .bind(v)
            .fetch_optional(&mut *conn)
            .await
            .map_err(kampus_db::classify)?
            .is_some(),
            None if overridden => sqlx::query(sql(format!(
                "update {} set raw = null, version = version + 1, updated_at = now()
                     where record_id = $1 and account_id = $2 and raw is not null returning 1",
                self.table("grade")
            )))
            .bind(total.id.as_uuid())
            .bind(person.as_uuid())
            .fetch_optional(&mut *conn)
            .await
            .map_err(kampus_db::classify)?
            .is_some(),
            None => sqlx::query(sql(format!(
                "delete from {} where record_id = $1 and account_id = $2 returning 1",
                self.table("grade")
            )))
            .bind(total.id.as_uuid())
            .bind(person.as_uuid())
            .fetch_optional(&mut *conn)
            .await
            .map_err(kampus_db::classify)?
            .is_some(),
        };
        if changed {
            self.publish_in(conn, events::TOTAL_CHANGED, course, total.id, Some(person), None)
                .await?;
        }
        let counted = if overridden {
            existing.and_then(|h| h.value)
        } else {
            value
        };
        Ok((counted, changed))
    }
}
