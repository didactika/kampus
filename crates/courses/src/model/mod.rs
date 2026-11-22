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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of courses: categories, courses, sections, groups and group sets, with their visibility
//! and the lifecycle of a course.

use std::str::FromStr;

use kampus_access::{PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Timestamp};
use serde::{Deserialize, Serialize};

kampus_common::define_id! {
    /// Identifies a course category.
    pub struct CategoryId;
}

kampus_common::define_id! {
    /// Identifies a course.
    pub struct CourseId;
}

kampus_common::define_id! {
    /// Identifies a section of a course.
    pub struct SectionId;
}

kampus_common::define_id! {
    /// Identifies a group of a course.
    pub struct GroupId;
}

kampus_common::define_id! {
    /// Identifies a group set of a course.
    pub struct GroupSetId;
}

/// Whether a node is shown. It is the node's own setting: what learners see also depends on every
/// ancestor, and nothing is ever copied down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Visibility {
    /// Shown.
    Shown,
    /// Hidden.
    Hidden,
    /// Shown inside a window; either end may be open.
    Scheduled {
        /// Shown from.
        from: Option<Timestamp>,
        /// Shown until.
        until: Option<Timestamp>,
    },
}

impl Visibility {
    /// The mode as stored.
    #[must_use]
    pub const fn mode(self) -> &'static str {
        match self {
            Self::Shown => "shown",
            Self::Hidden => "hidden",
            Self::Scheduled { .. } => "scheduled",
        }
    }

    /// The window, as stored.
    #[must_use]
    pub const fn window(self) -> (Option<Timestamp>, Option<Timestamp>) {
        match self {
            Self::Scheduled { from, until } => (from, until),
            Self::Shown | Self::Hidden => (None, None),
        }
    }

    /// Reads a stored visibility.
    #[must_use]
    pub fn read(mode: &str, from: Option<Timestamp>, until: Option<Timestamp>) -> Self {
        match mode {
            "hidden" => Self::Hidden,
            "scheduled" => Self::Scheduled { from, until },
            _ => Self::Shown,
        }
    }

    /// Whether the node itself is shown at `now`.
    #[must_use]
    pub fn shown_at(self, now: Timestamp) -> bool {
        match self {
            Self::Shown => true,
            Self::Hidden => false,
            Self::Scheduled { from, until } => from.is_none_or(|f| f <= now) && until.is_none_or(|u| now < u),
        }
    }

    /// Checks a window: its end after its start.
    ///
    /// # Errors
    ///
    /// `validation` (`courses.visibility_invalid`) for a window that ends before it starts.
    pub fn validate(self) -> Result<Self, Error> {
        if let Self::Scheduled {
            from: Some(from),
            until: Some(until),
        } = self
            && until <= from
        {
            return Err(Error::validation(
                "courses.visibility_invalid",
                vec![FieldError::new("visibility.until", "after_from")],
            ));
        }
        Ok(self)
    }
}

/// The lifecycle state of a course.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CourseState {
    /// Being prepared.
    Draft,
    /// Open to its participants.
    Published,
    /// Kept read-only: its scope is frozen.
    Archived,
    /// Waiting for its deletion at the end of the grace period.
    DeletionScheduled,
}

impl CourseState {
    /// The state as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Archived => "archived",
            Self::DeletionScheduled => "deletion_scheduled",
        }
    }
}

impl FromStr for CourseState {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "archived" => Ok(Self::Archived),
            "deletion_scheduled" => Ok(Self::DeletionScheduled),
            _ => Err(Error::validation(
                "courses.state_invalid",
                vec![FieldError::new("to", "one_of")],
            )),
        }
    }
}

/// Who sees a group and its members.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GroupVisibility {
    /// Everyone in the course.
    Everyone,
    /// Its members.
    Members,
    /// Each member sees only themselves.
    OwnOnly,
    /// Nobody but those who manage groups.
    Nobody,
}

impl GroupVisibility {
    /// The value as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Everyone => "everyone",
            Self::Members => "members",
            Self::OwnOnly => "own_only",
            Self::Nobody => "nobody",
        }
    }

    /// Reads a stored value.
    #[must_use]
    pub fn read(text: &str) -> Self {
        match text {
            "members" => Self::Members,
            "own_only" => Self::OwnOnly,
            "nobody" => Self::Nobody,
            _ => Self::Everyone,
        }
    }

    /// Whether a group may be used for group mode: its members must be able to see it.
    #[must_use]
    pub const fn allows_group_mode(self) -> bool {
        matches!(self, Self::Everyone | Self::Members)
    }
}

/// Where a group membership comes from; a module removes only what it added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "module", rename_all = "snake_case")]
pub enum MemberOrigin {
    /// Added by a person.
    Manual,
    /// Added by a module, such as enrolment.
    Module(String),
}

/// A course category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Category {
    /// The category.
    pub id: CategoryId,
    /// Its scope.
    pub scope: ScopeId,
    /// Its campus.
    pub campus: ScopeId,
    /// Its parent; the campus itself when absent.
    pub parent: Option<CategoryId>,
    /// Its code, unique in the campus when set.
    pub code: Option<String>,
    /// Its name.
    pub name: String,
    /// Its description.
    pub description: Option<String>,
    /// Its place among its siblings.
    pub sort_key: String,
    /// Its own visibility.
    pub visibility: Visibility,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// The fields of a category a manager edits.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct CategoryFields {
    /// Its name.
    pub name: String,
    /// Its code.
    #[serde(default)]
    pub code: Option<String>,
    /// Its description.
    #[serde(default)]
    pub description: Option<String>,
    /// Its own visibility.
    #[serde(default = "shown")]
    pub visibility: Visibility,
}

/// The default visibility.
const fn shown() -> Visibility {
    Visibility::Shown
}

/// A course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Course {
    /// The course.
    pub id: CourseId,
    /// Its scope.
    pub scope: ScopeId,
    /// Its campus.
    pub campus: ScopeId,
    /// Its category.
    pub category: CategoryId,
    /// Its short name, unique in the campus.
    pub short_name: String,
    /// Its full name.
    pub full_name: String,
    /// Its code, unique in the campus when set.
    pub code: Option<String>,
    /// Its summary.
    pub summary: Option<String>,
    /// The layout plugin that presents it.
    pub layout: String,
    /// The language it is forced to, when set.
    pub language_code: Option<String>,
    /// When it starts.
    pub starts_at: Option<Timestamp>,
    /// When it ends.
    pub ends_at: Option<Timestamp>,
    /// Its lifecycle state.
    pub state: CourseState,
    /// When a scheduled deletion runs.
    pub deletion_due_at: Option<Timestamp>,
    /// Its own visibility.
    pub visibility: Visibility,
    /// Its place in its category.
    pub sort_key: String,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// The fields of a course its teachers edit.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct CourseFields {
    /// Its short name.
    pub short_name: String,
    /// Its full name.
    pub full_name: String,
    /// Its code.
    #[serde(default)]
    pub code: Option<String>,
    /// Its summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// The layout plugin; the default layout when absent.
    #[serde(default)]
    pub layout: Option<String>,
    /// The language it is forced to.
    #[serde(default)]
    pub language_code: Option<String>,
    /// When it starts.
    #[serde(default)]
    pub starts_at: Option<Timestamp>,
    /// When it ends.
    #[serde(default)]
    pub ends_at: Option<Timestamp>,
    /// Its own visibility.
    #[serde(default = "shown")]
    pub visibility: Visibility,
}

/// A section of a course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Section {
    /// The section.
    pub id: SectionId,
    /// Its scope.
    pub scope: ScopeId,
    /// Its course.
    pub course: CourseId,
    /// The section it is nested in.
    pub parent: Option<SectionId>,
    /// Its title.
    pub title: Option<String>,
    /// Its summary.
    pub summary: Option<String>,
    /// Its place among its siblings.
    pub sort_key: String,
    /// Its own visibility.
    pub visibility: Visibility,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// The fields of a section its teachers edit.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct SectionFields {
    /// Its title.
    #[serde(default)]
    pub title: Option<String>,
    /// Its summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// Its own visibility.
    #[serde(default = "shown")]
    pub visibility: Visibility,
}

/// A group of a course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Group {
    /// The group.
    pub id: GroupId,
    /// Its course.
    pub course: CourseId,
    /// The section it is limited to.
    pub section: Option<SectionId>,
    /// Its name, unique in the course.
    pub name: String,
    /// Its code, unique in the course when set.
    pub code: Option<String>,
    /// Its description.
    pub description: Option<String>,
    /// Who sees it.
    pub visibility: GroupVisibility,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// The fields of a group.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct GroupFields {
    /// Its name.
    pub name: String,
    /// Its code.
    #[serde(default)]
    pub code: Option<String>,
    /// Its description.
    #[serde(default)]
    pub description: Option<String>,
    /// The section it is limited to.
    #[serde(default)]
    pub section: Option<SectionId>,
    /// Who sees it.
    #[serde(default = "everyone")]
    pub visibility: GroupVisibility,
}

/// The default visibility of a group.
const fn everyone() -> GroupVisibility {
    GroupVisibility::Everyone
}

/// A member of a group, once per origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Member {
    /// The account.
    pub account: PrincipalId,
    /// Where the membership comes from.
    pub origin: MemberOrigin,
}

/// A group set of a course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct GroupSet {
    /// The set.
    pub id: GroupSetId,
    /// Its course.
    pub course: CourseId,
    /// Its name, unique in the course.
    pub name: String,
    /// Its code.
    pub code: Option<String>,
    /// Its groups.
    pub groups: Vec<GroupId>,
}

/// Where a node goes among its siblings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(tag = "at", content = "sibling", rename_all = "snake_case")]
pub enum Place<T> {
    /// First.
    First,
    /// Last.
    Last,
    /// Right after a sibling.
    After(T),
    /// Right before a sibling.
    Before(T),
}

#[allow(
    clippy::derivable_impls,
    reason = "a derive would require every identifier to have a default"
)]
impl<T> Default for Place<T> {
    fn default() -> Self {
        Self::Last
    }
}

/// Whether people of a course, a section or an activity work in groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupMode {
    /// No groups.
    None,
    /// Each group works on its own and sees only itself.
    Separate,
    /// Each group works on its own and sees the others.
    Visible,
}
