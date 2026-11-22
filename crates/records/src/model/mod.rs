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

//! The entities of records: the categories and grade records of a course, the entries that change the
//! grade of a person, and the grades themselves. Numbers are exact decimals; rounding is for display.

use kampus_access::PrincipalId;
use kampus_activities::ModuleId;
use kampus_common::Timestamp;
use kampus_courses::CourseId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

kampus_common::define_id! {
    /// Identifies a grade category.
    pub struct CategoryId;
}

kampus_common::define_id! {
    /// Identifies a grade record.
    pub struct RecordId;
}

kampus_common::define_id! {
    /// Identifies a grade entry.
    pub struct EntryId;
}

kampus_common::define_id! {
    /// Identifies a version of a scale.
    pub struct ScaleId;
}

kampus_common::define_id! {
    /// Identifies a version of a letter set.
    pub struct LetterSetId;
}

/// How a category aggregates its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    /// The mean of the grades.
    Mean,
    /// The mean weighted by the weight of each record.
    WeightedMean,
    /// The mean weighted by the maximum of each record.
    SimpleWeightedMean,
    /// The sum of the grades.
    Natural,
    /// The median.
    Median,
    /// The lowest.
    Lowest,
    /// The highest.
    Highest,
    /// The most frequent.
    Mode,
}

impl Method {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::WeightedMean => "weighted_mean",
            Self::SimpleWeightedMean => "simple_weighted_mean",
            Self::Natural => "natural",
            Self::Median => "median",
            Self::Lowest => "lowest",
            Self::Highest => "highest",
            Self::Mode => "mode",
        }
    }

    /// The method a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        [
            Self::Mean,
            Self::WeightedMean,
            Self::SimpleWeightedMean,
            Self::Median,
            Self::Lowest,
            Self::Highest,
            Self::Mode,
        ]
        .into_iter()
        .find(|m| m.code() == code)
        .unwrap_or(Self::Natural)
    }
}

/// A node of the tree of records of a course, which aggregates its children.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Category {
    /// The category.
    pub id: CategoryId,
    /// Its course.
    pub course: CourseId,
    /// Its parent; none for the root of the course.
    pub parent: Option<CategoryId>,
    /// Its name.
    pub name: String,
    /// How it aggregates.
    pub method: Method,
    /// How many of the lowest grades it drops.
    pub drop_lowest: i32,
    /// How many of the highest grades it keeps; zero keeps all.
    pub keep_highest: i32,
    /// Whether an empty grade is left out instead of counted as zero.
    pub exclude_empty: bool,
    /// Its place among its siblings.
    pub sort_key: String,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// What a person sets on a category.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct CategoryFields {
    /// Its name.
    pub name: String,
    /// How it aggregates.
    #[serde(default = "natural")]
    pub method: Method,
    /// How many of the lowest grades it drops.
    #[serde(default)]
    pub drop_lowest: i32,
    /// How many of the highest grades it keeps; zero keeps all.
    #[serde(default)]
    pub keep_highest: i32,
    /// Whether an empty grade is left out instead of counted as zero.
    #[serde(default = "yes")]
    pub exclude_empty: bool,
}

/// The default method of a category.
const fn natural() -> Method {
    Method::Natural
}

/// The default of the flags that are on unless said otherwise.
const fn yes() -> bool {
    true
}

/// What a grade record stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    /// A grade an activity gives.
    Activity,
    /// A grade a teacher enters.
    Manual,
    /// The total of a category.
    CategoryTotal,
    /// The total of the course.
    CourseTotal,
}

impl RecordKind {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Manual => "manual",
            Self::CategoryTotal => "category_total",
            Self::CourseTotal => "course_total",
        }
    }

    /// The kind a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        [Self::Manual, Self::CategoryTotal, Self::CourseTotal]
            .into_iter()
            .find(|k| k.code() == code)
            .unwrap_or(Self::Activity)
    }
}

/// What a grade is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GradeType {
    /// A number between the minimum and the maximum.
    Value,
    /// An item of a scale.
    Scale,
    /// Free text.
    Text,
}

impl GradeType {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Scale => "scale",
            Self::Text => "text",
        }
    }

    /// The type a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        [Self::Scale, Self::Text]
            .into_iter()
            .find(|t| t.code() == code)
            .unwrap_or(Self::Value)
    }
}

/// One graded thing of a course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Record {
    /// The record.
    pub id: RecordId,
    /// Its course.
    pub course: CourseId,
    /// What it stands for.
    pub kind: RecordKind,
    /// Its title.
    pub title: String,
    /// The course module it grades, for an activity record.
    pub course_module: Option<ModuleId>,
    /// Which of the grades of its course module it is.
    pub item_number: Option<i32>,
    /// The category that aggregates it; none for the course total.
    pub category: Option<CategoryId>,
    /// The category it totals, for a total.
    pub total_of: Option<CategoryId>,
    /// What its grades are made of.
    pub grade_type: GradeType,
    /// The version of the scale its grades are items of, for a scale record.
    pub scale: Option<ScaleId>,
    /// The lowest grade.
    pub min: Decimal,
    /// The highest grade.
    pub max: Decimal,
    /// The grade that passes.
    pub pass_grade: Option<Decimal>,
    /// Its weight in the category.
    pub weight: Decimal,
    /// Whether it counts on top of the maximum of its category.
    pub extra_credit: bool,
    /// What its grades are multiplied by.
    pub multiplier: Decimal,
    /// What is added to its grades after multiplying.
    pub offset: Decimal,
    /// Hidden until then.
    pub hidden_until: Option<Timestamp>,
    /// Locked since then.
    pub locked_at: Option<Timestamp>,
    /// Its place among its siblings.
    pub sort_key: String,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// What a person sets on a grade record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct RecordFields {
    /// Its title.
    pub title: String,
    /// What its grades are made of.
    #[serde(default = "value")]
    pub grade_type: GradeType,
    /// The scale its grades are items of; required for a scale record, whose range is the items.
    #[serde(default)]
    pub scale: Option<ScaleId>,
    /// The lowest grade.
    #[serde(default)]
    pub min: Decimal,
    /// The highest grade.
    #[serde(default = "hundred")]
    pub max: Decimal,
    /// The grade that passes.
    #[serde(default)]
    pub pass_grade: Option<Decimal>,
    /// Its weight in the category.
    #[serde(default = "one")]
    pub weight: Decimal,
    /// Whether it counts on top of the maximum of its category.
    #[serde(default)]
    pub extra_credit: bool,
    /// What its grades are multiplied by.
    #[serde(default = "one")]
    pub multiplier: Decimal,
    /// What is added to its grades after multiplying.
    #[serde(default)]
    pub offset: Decimal,
    /// Hidden until then.
    #[serde(default)]
    pub hidden_until: Option<Timestamp>,
}

/// The default type of a grade.
const fn value() -> GradeType {
    GradeType::Value
}

/// The default maximum of a record.
const fn hundred() -> Decimal {
    Decimal::ONE_HUNDRED
}

/// The default weight and multiplier.
const fn one() -> Decimal {
    Decimal::ONE
}

impl RecordFields {
    /// The fields of a record that grades from zero to a maximum, with the usual weight.
    #[must_use]
    pub fn new(title: &str, max: Decimal) -> Self {
        Self {
            title: title.to_owned(),
            grade_type: GradeType::Value,
            scale: None,
            min: Decimal::ZERO,
            max,
            pass_grade: None,
            weight: Decimal::ONE,
            extra_credit: false,
            multiplier: Decimal::ONE,
            offset: Decimal::ZERO,
            hidden_until: None,
        }
    }
}

/// Where the change of a grade came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The activity the record grades.
    Activity,
    /// A teacher.
    Teacher,
    /// An import.
    Import,
    /// A formula.
    Formula,
}

impl Source {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Teacher => "teacher",
            Self::Import => "import",
            Self::Formula => "formula",
        }
    }

    /// The source a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        [Self::Teacher, Self::Import, Self::Formula]
            .into_iter()
            .find(|s| s.code() == code)
            .unwrap_or(Self::Activity)
    }
}

/// The current grade of a person on a record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Grade {
    /// The record.
    pub record: RecordId,
    /// The person.
    pub account: PrincipalId,
    /// What was graded, as given.
    pub raw: Option<Decimal>,
    /// What counts: the override when there is one, otherwise the raw grade after the multiplier and offset.
    #[serde(rename = "final")]
    pub final_value: Option<Decimal>,
    /// The text of a text grade.
    pub text: Option<String>,
    /// Whether a teacher overrode it.
    pub overridden: bool,
    /// Whether it is left out of the totals.
    pub excluded: bool,
    /// Hidden until then.
    pub hidden_until: Option<Timestamp>,
    /// Locked since then.
    pub locked_at: Option<Timestamp>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// An immutable change to the grade of a person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Entry {
    /// The entry.
    pub id: EntryId,
    /// The record.
    pub record: RecordId,
    /// The person.
    pub account: PrincipalId,
    /// The grade it set.
    pub raw: Option<Decimal>,
    /// The text it set.
    pub text: Option<String>,
    /// What the grader said about it.
    pub feedback: Option<String>,
    /// Where it came from.
    pub source: Source,
    /// Who made it, when a person did.
    pub actor: Option<uuid::Uuid>,
    /// When.
    pub at: Timestamp,
}

/// A grade a source gives to a person.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct GradeWrite {
    /// The person.
    pub account: PrincipalId,
    /// The grade, for a numeric record.
    #[serde(default)]
    pub raw: Option<Decimal>,
    /// The text, for a text record.
    #[serde(default)]
    pub text: Option<String>,
    /// What the grader says about it.
    #[serde(default)]
    pub feedback: Option<String>,
}

/// What happened to a grade a source wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct GradeChange {
    /// The grade after it.
    pub grade: Grade,
    /// Whether it changed anything; the same grade sent twice changes nothing.
    pub changed: bool,
}

/// The records of a course: its categories and its grade records, each in its place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Tree {
    /// The categories, the root first.
    pub categories: Vec<Category>,
    /// The grade records.
    pub records: Vec<Record>,
}

/// An ordered list of items used instead of numbers; a new version is a new row, and a record keeps the
/// version it was graded with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Scale {
    /// The version of the scale.
    pub id: ScaleId,
    /// The scope it is defined in; it is available there and below.
    pub scope: kampus_access::ScopeId,
    /// Its code, the same across versions.
    pub code: String,
    /// Its version.
    pub version: i32,
    /// Its items, from the lowest to the highest.
    pub items: Vec<String>,
}

/// One band of a letter set: from this percentage up, the grade is this letter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Band {
    /// The lowest percentage of the band.
    pub min_percent: Decimal,
    /// The letter.
    pub letter: String,
}

/// Bands that turn a percentage into a letter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct LetterSet {
    /// The version of the set.
    pub id: LetterSetId,
    /// The scope it is defined in; it applies there and below.
    pub scope: kampus_access::ScopeId,
    /// Its version.
    pub version: i32,
    /// Its bands, from the highest percentage down.
    pub bands: Vec<Band>,
}

kampus_common::define_id! {
    /// Identifies an export of the grades of a course.
    pub struct ExportId;
}

/// An export of the grades of a course, as a file for the person who asked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Export {
    /// The export.
    pub id: ExportId,
    /// The course.
    pub course: CourseId,
    /// Whether it also has the people who no longer take part.
    pub include_inactive: bool,
    /// `pending`, `completed`, `failed` or `expired`.
    pub state: String,
    /// The file, while it is available.
    pub file: Option<uuid::Uuid>,
    /// Why it failed.
    pub error: Option<String>,
    /// When it was asked for.
    pub created_at: Timestamp,
    /// When it was made.
    pub completed_at: Option<Timestamp>,
    /// When the file goes.
    pub expires_at: Option<Timestamp>,
}
