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

//! External references of courses: the identifiers other systems give categories, courses, sections,
//! groups and group sets, one per system and record, unique in the campus for each kind. They are set
//! with the permission that manages the record, and an integration finds a record by them or creates or
//! updates a category or a course by them.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use kampus_db::references::{ExternalReference, References};
use serde::{Deserialize, Serialize};

use crate::permissions::{CATEGORY_MANAGE, COURSE_UPDATE, GROUPS_MANAGE, SECTION_MANAGE};
use crate::{
    Category, CategoryFields, CategoryId, Course, CourseFields, CourseId, Courses, GroupId, Place, SectionId,
    sql,
};

/// A kind of record of courses that carries external references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    /// A category.
    Category,
    /// A course.
    Course,
    /// A section.
    Section,
    /// A group.
    Group,
    /// A group set.
    GroupSet,
}

impl RecordKind {
    /// Every kind.
    pub const ALL: [Self; 5] = [
        Self::Category,
        Self::Course,
        Self::Section,
        Self::Group,
        Self::GroupSet,
    ];

    /// The table of its references.
    const fn table(self) -> &'static str {
        match self {
            Self::Category => "category_reference",
            Self::Course => "course_reference",
            Self::Section => "section_reference",
            Self::Group => "group_reference",
            Self::GroupSet => "group_set_reference",
        }
    }

    /// The permission that manages its records, and so their references.
    const fn permission(self) -> &'static str {
        match self {
            Self::Category => CATEGORY_MANAGE,
            Self::Course => COURSE_UPDATE,
            Self::Section => SECTION_MANAGE,
            Self::Group | Self::GroupSet => GROUPS_MANAGE,
        }
    }

    /// The segment of its records in the routes, such as `group-sets`.
    #[must_use]
    pub const fn segment(self) -> &'static str {
        match self {
            Self::Category => "categories",
            Self::Course => "courses",
            Self::Section => "sections",
            Self::Group => "groups",
            Self::GroupSet => "group-sets",
        }
    }
}

impl Courses {
    /// The references of a kind of record.
    pub(crate) fn references(&self, kind: RecordKind) -> References {
        References::new(&self.db, crate::module::NAME, kind.table())
    }

    /// The campus of a record and the scope its permission is checked in.
    async fn place_of(&self, kind: RecordKind, id: uuid::Uuid) -> Result<(ScopeId, ScopeId)> {
        let course_of = |course: Course| (course.campus, course.scope);
        Ok(match kind {
            RecordKind::Category => {
                let category = self.category(CategoryId::from_uuid(id)).await?;
                (category.campus, category.scope)
            }
            RecordKind::Course => course_of(self.course(CourseId::from_uuid(id)).await?),
            RecordKind::Section => {
                let section = self.section(SectionId::from_uuid(id)).await?;
                (self.course(section.course).await?.campus, section.scope)
            }
            RecordKind::Group => course_of(
                self.course(self.group(GroupId::from_uuid(id)).await?.course)
                    .await?,
            ),
            RecordKind::GroupSet => {
                let course: uuid::Uuid = sqlx::query_scalar(sql(format!(
                    "select course_id from {} where id = $1",
                    self.table("group_set")
                )))
                .bind(id)
                .fetch_optional(self.db.pool())
                .await
                .map_err(kampus_db::classify)?
                .ok_or_else(|| Error::not_found("courses.group_set_not_found"))?;
                course_of(self.course(CourseId::from_uuid(course)).await?)
            }
        })
    }

    /// The identifiers systems give a record; needs the permission that manages it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown record, `permission_denied` without the permission.
    pub async fn external_references(
        &self,
        actor: &Actor,
        kind: RecordKind,
        id: uuid::Uuid,
    ) -> Result<Vec<ExternalReference>> {
        let (_, scope) = self.place_of(kind, id).await?;
        self.access.require(actor, kind.permission(), scope).await?;
        self.references(kind).of(id).await
    }

    /// Gives a record the identifier a system uses for it, replacing the one that system gave it before;
    /// needs the permission that manages it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown record, `permission_denied` without the permission, `conflict`
    /// (`references.taken`) when another record of the kind has that identifier in the campus.
    pub async fn set_external_reference(
        &self,
        actor: &Actor,
        kind: RecordKind,
        id: uuid::Uuid,
        reference: &ExternalReference,
    ) -> Result<Vec<ExternalReference>> {
        let (campus, scope) = self.place_of(kind, id).await?;
        self.access.require(actor, kind.permission(), scope).await?;
        let references = self.references(kind);
        references.set(campus.as_uuid(), id, reference).await?;
        references.of(id).await
    }

    /// Removes the identifier a system gave a record; needs the permission that manages it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown record or one the system gave no identifier, `permission_denied`
    /// without the permission.
    pub async fn remove_external_reference(
        &self,
        actor: &Actor,
        kind: RecordKind,
        id: uuid::Uuid,
        system: &str,
    ) -> Result<()> {
        let (_, scope) = self.place_of(kind, id).await?;
        self.access.require(actor, kind.permission(), scope).await?;
        if self.references(kind).remove(id, system).await? {
            Ok(())
        } else {
            Err(Error::not_found("references.not_found"))
        }
    }

    /// The record of a kind a system knows by an identifier in a campus.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn find_by_reference(
        &self,
        kind: RecordKind,
        campus: ScopeId,
        reference: &ExternalReference,
    ) -> Result<Option<uuid::Uuid>> {
        self.references(kind).find(campus.as_uuid(), reference).await
    }

    /// Updates the category a system knows by an identifier, or creates it with that identifier last
    /// under its parent; returns it and whether it was created.
    ///
    /// # Errors
    ///
    /// Those of [`Courses::create_category`] and [`Courses::update_category`].
    pub async fn upsert_category_by_reference(
        &self,
        actor: &Actor,
        campus: ScopeId,
        reference: &ExternalReference,
        parent: Option<CategoryId>,
        fields: CategoryFields,
    ) -> Result<(Category, bool)> {
        match self
            .find_by_reference(RecordKind::Category, campus, reference)
            .await?
        {
            Some(id) => Ok((
                self.update_category(actor, CategoryId::from_uuid(id), fields, None)
                    .await?,
                false,
            )),
            None => Ok((
                self.create_category_with(actor, (campus, parent), fields, Place::Last, Some(reference))
                    .await?,
                true,
            )),
        }
    }

    /// Updates the course a system knows by an identifier in the campus of a category, or creates it with
    /// that identifier last in the category; returns it and whether it was created. A course found in
    /// another category stays there.
    ///
    /// # Errors
    ///
    /// Those of [`Courses::create_course`] and [`Courses::update_course`].
    pub async fn upsert_course_by_reference(
        &self,
        actor: &Actor,
        category: CategoryId,
        reference: &ExternalReference,
        fields: CourseFields,
    ) -> Result<(Course, bool)> {
        let campus = self.category(category).await?.campus;
        match self
            .find_by_reference(RecordKind::Course, campus, reference)
            .await?
        {
            Some(id) => Ok((
                self.update_course(actor, CourseId::from_uuid(id), fields, None)
                    .await?,
                false,
            )),
            None => Ok((
                self.create_course_with(actor, category, fields, Place::Last, Some(reference))
                    .await?,
                true,
            )),
        }
    }
}
