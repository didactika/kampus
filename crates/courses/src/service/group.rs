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

//! Groups and group sets of a course: only participants are members, a group limited to a section takes
//! only people who see it, and a membership a module added is removed only by that module.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::{check_version, optional, required, taken, valid};
use crate::permissions::{GROUPS_ACCESS_ALL, GROUPS_MANAGE};
use crate::{
    Course, CourseId, Courses, Group, GroupFields, GroupId, GroupSet, GroupSetId, GroupVisibility, Member,
    MemberOrigin, SectionId, sql,
};

/// The columns of a group, as read.
const COLUMNS: &str = "id, course_id, section_id, name, code, description, visibility, version";

/// The origin of a membership added by a person.
const MANUAL: &str = "manual";

/// A group from its row.
fn group(row: &PgRow) -> Group {
    Group {
        id: GroupId::from_uuid(row.get("id")),
        course: CourseId::from_uuid(row.get("course_id")),
        section: row
            .get::<Option<uuid::Uuid>, _>("section_id")
            .map(SectionId::from_uuid),
        name: row.get("name"),
        code: row.get("code"),
        description: row.get("description"),
        visibility: GroupVisibility::read(row.get::<String, _>("visibility").as_str()),
        version: row.get("version"),
    }
}

/// Checks the fields of a group, every problem at once.
fn checked(fields: GroupFields) -> Result<GroupFields> {
    let mut problems = Vec::new();
    let name = required("name", &fields.name, &mut problems);
    valid("courses.group_invalid", problems)?;
    Ok(GroupFields {
        name,
        code: optional(fields.code),
        description: optional(fields.description),
        section: fields.section,
        visibility: fields.visibility,
    })
}

/// Checks the name of a module that owns memberships.
fn module_origin(module: &str) -> Result<&str> {
    let valid = module != MANUAL
        && module.starts_with(|c: char| c.is_ascii_lowercase())
        && module
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if valid {
        Ok(module)
    } else {
        Err(Error::validation(
            "courses.member_origin_invalid",
            vec![FieldError::new("module", "name")],
        ))
    }
}

/// A section of another course answered as a field error.
fn section_of_course(error: Error) -> Error {
    if error.code() == "db.foreign_key_violation" {
        Error::validation(
            "courses.group_section_invalid",
            vec![FieldError::new("section", "course")],
        )
    } else {
        error
    }
}

impl Courses {
    /// Creates a group in a course, limited to one of its sections when given; needs
    /// `courses.groups.manage` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields or a section of another
    /// course, `conflict` for a name or code in use in the course, or (`customise.feature_off`) where groups
    /// are turned off.
    pub async fn create_group(&self, actor: &Actor, course: CourseId, fields: GroupFields) -> Result<Group> {
        let fields = checked(fields)?;
        let course = self.course(course).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        self.require_feature(crate::settings::customise::GROUPS_FEATURE, course.scope)
            .await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, course_id, section_id, name, code, description, visibility)
             values ($1, $2, $3, $4, $5, $6, $7) returning {COLUMNS}",
            self.table("group")
        )))
        .bind(GroupId::generate().as_uuid())
        .bind(course.id.as_uuid())
        .bind(fields.section.map(|s| s.as_uuid()))
        .bind(&fields.name)
        .bind(&fields.code)
        .bind(&fields.description)
        .bind(fields.visibility.as_str())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(section_of_course)
        .map_err(taken("courses.group_name_taken"))?;
        Ok(group(&row))
    }

    /// A group, as stored.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group.
    pub async fn group(&self, id: GroupId) -> Result<Group> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.read_group_in(&mut conn, id, false).await
    }

    /// The groups of a course the actor sees: every group with `courses.groups.access_all`; otherwise the
    /// groups shown to everyone, and those the actor belongs to unless they are shown to nobody.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course.
    pub async fn groups(&self, actor: &Actor, course: CourseId) -> Result<Vec<Group>> {
        let course = self.course(course).await?;
        let all = self
            .access
            .check(actor, GROUPS_ACCESS_ALL, course.scope)
            .await?
            .allowed;
        let me = match actor {
            Actor::Principal(p) => Some(p.as_uuid()),
            _ => None,
        };
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {groups} g where course_id = $1 and ($2 or visibility = 'everyone'
             or (visibility in ('members', 'own_only') and exists (
                 select 1 from {members} m where m.group_id = g.id and m.account_id = $3)))
             order by lower(name), id",
            groups = self.table("group"),
            members = self.table("group_member")
        )))
        .bind(course.id.as_uuid())
        .bind(all)
        .bind(me)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(group).collect())
    }

    /// Changes the fields of a group; needs `courses.groups.manage` in its course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields,
    /// `precondition_failed` when it changed since it was read, `conflict` for a name or code in use.
    pub async fn update_group(
        &self,
        actor: &Actor,
        id: GroupId,
        fields: GroupFields,
        expected_version: Option<i32>,
    ) -> Result<Group> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let stored = self.read_group_in(&mut tx, id, true).await?;
        let course = self.read_course_in(&mut tx, stored.course, false).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        check_version(expected_version, stored.version, "courses.group_changed")?;
        let row = sqlx::query(sql(format!(
            "update {} set section_id = $2, name = $3, code = $4, description = $5, visibility = $6,
             version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("group")
        )))
        .bind(id.as_uuid())
        .bind(fields.section.map(|s| s.as_uuid()))
        .bind(&fields.name)
        .bind(&fields.code)
        .bind(&fields.description)
        .bind(fields.visibility.as_str())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(section_of_course)
        .map_err(taken("courses.group_name_taken"))?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(group(&row))
    }

    /// Deletes a group with its memberships; needs `courses.groups.manage` in its course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `precondition_failed` when it changed since it was
    /// read.
    pub async fn delete_group(
        &self,
        actor: &Actor,
        id: GroupId,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.read_group_in(&mut tx, id, true).await?;
        let course = self.read_course_in(&mut tx, stored.course, false).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        check_version(expected_version, stored.version, "courses.group_changed")?;
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("group"))))
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Checks that a group may be chosen for group mode: its members must be able to see it.
    ///
    /// # Errors
    ///
    /// `validation` (`courses.group_mode_group_invalid`) for a group shown to each member only or to
    /// nobody.
    pub fn check_group_mode_group(group: &Group) -> Result<()> {
        if group.visibility.allows_group_mode() {
            Ok(())
        } else {
            Err(Error::validation(
                "courses.group_mode_group_invalid",
                vec![FieldError::new("group", "visibility")],
            ))
        }
    }

    /// Adds a person to a group by hand; needs `courses.groups.manage` in its course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` (`courses.member_not_participant`) for
    /// someone who does not take part in the course or does not see the group's section.
    pub async fn add_member(&self, actor: &Actor, group: GroupId, account: PrincipalId) -> Result<()> {
        let (group, course) = self.group_and_course(group).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        self.insert_member(&group, &course, account, MANUAL).await
    }

    /// Removes the membership a person added by hand; one a module added stays.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `courses.groups.manage`.
    pub async fn remove_member(&self, actor: &Actor, group: GroupId, account: PrincipalId) -> Result<()> {
        let (group, course) = self.group_and_course(group).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        self.delete_member(group.id, account, MANUAL).await
    }

    /// Adds a membership that a module owns on behalf of an actor, such as a rule acting as its owner;
    /// needs `courses.groups.manage` in the course of the group, and only that module removes it.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an invalid module name or someone who
    /// does not take part in the course.
    pub async fn add_member_for(
        &self,
        actor: &Actor,
        module: &str,
        group: GroupId,
        account: PrincipalId,
    ) -> Result<()> {
        let origin = module_origin(module)?;
        let (group, course) = self.group_and_course(group).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        self.insert_member(&group, &course, account, origin).await
    }

    /// Adds a membership that a module owns, such as enrolment; only that module removes it.
    ///
    /// # Errors
    ///
    /// `validation` for an invalid module name or someone who does not take part in the course.
    pub async fn add_module_member(&self, module: &str, group: GroupId, account: PrincipalId) -> Result<()> {
        let origin = module_origin(module)?;
        let (group, course) = self.group_and_course(group).await?;
        self.insert_member(&group, &course, account, origin).await
    }

    /// Removes a membership a module owns.
    ///
    /// # Errors
    ///
    /// `validation` for an invalid module name.
    pub async fn remove_module_member(
        &self,
        module: &str,
        group: GroupId,
        account: PrincipalId,
    ) -> Result<()> {
        let origin = module_origin(module)?;
        self.delete_member(group, account, origin).await
    }

    /// The members of a group the actor sees: all of them with `courses.groups.access_all`, or when the
    /// group is shown to everyone, or to its members and the actor is one; only the actor when it is shown
    /// to each member only.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group.
    pub async fn members(&self, actor: &Actor, group: GroupId) -> Result<Vec<Member>> {
        let (group, course) = self.group_and_course(group).await?;
        let rows = sqlx::query(sql(format!(
            "select account_id, origin from {} where group_id = $1 order by added_at, account_id, origin",
            self.table("group_member")
        )))
        .bind(group.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let members: Vec<Member> = rows
            .iter()
            .map(|r| Member {
                account: PrincipalId::from_uuid(r.get("account_id")),
                origin: match r.get::<String, _>("origin") {
                    o if o == MANUAL => MemberOrigin::Manual,
                    module => MemberOrigin::Module(module),
                },
            })
            .collect();
        if self
            .access
            .check(actor, GROUPS_ACCESS_ALL, course.scope)
            .await?
            .allowed
        {
            return Ok(members);
        }
        let me = match actor {
            Actor::Principal(p) => Some(*p),
            _ => None,
        };
        let member = members.iter().any(|m| Some(m.account) == me);
        Ok(match group.visibility {
            GroupVisibility::Everyone => members,
            GroupVisibility::Members if member => members,
            GroupVisibility::OwnOnly => members.into_iter().filter(|m| Some(m.account) == me).collect(),
            GroupVisibility::Members | GroupVisibility::Nobody => Vec::new(),
        })
    }

    /// Creates a group set in a course; needs `courses.groups.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an empty name, `conflict` for a name
    /// in use in the course.
    pub async fn create_group_set(
        &self,
        actor: &Actor,
        course: CourseId,
        name: &str,
        code: Option<String>,
    ) -> Result<GroupSet> {
        let mut problems = Vec::new();
        let name = required("name", name, &mut problems);
        valid("courses.group_set_invalid", problems)?;
        let course = self.course(course).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        let id = GroupSetId::generate();
        let code = optional(code);
        sqlx::query(sql(format!(
            "insert into {} (id, course_id, name, code) values ($1, $2, $3, $4)",
            self.table("group_set")
        )))
        .bind(id.as_uuid())
        .bind(course.id.as_uuid())
        .bind(&name)
        .bind(&code)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(taken("courses.group_set_name_taken"))?;
        Ok(GroupSet {
            id,
            course: course.id,
            name,
            code,
            groups: Vec::new(),
        })
    }

    /// Puts a group of the same course in a set, once; needs `courses.groups.manage` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for a group of another course.
    pub async fn add_to_set(&self, actor: &Actor, set: GroupSetId, group: GroupId) -> Result<()> {
        let course = self.set_course(set).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        sqlx::query(sql(format!(
            "insert into {} (set_id, group_id, course_id) values ($1, $2, $3) on conflict do nothing",
            self.table("group_set_group")
        )))
        .bind(set.as_uuid())
        .bind(group.as_uuid())
        .bind(course.id.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.foreign_key_violation" {
                Error::validation(
                    "courses.group_set_group_invalid",
                    vec![FieldError::new("group", "course")],
                )
            } else {
                e
            }
        })
        .map(drop)
    }

    /// Takes a group out of a set; needs `courses.groups.manage` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn remove_from_set(&self, actor: &Actor, set: GroupSetId, group: GroupId) -> Result<()> {
        let course = self.set_course(set).await?;
        self.access.require(actor, GROUPS_MANAGE, course.scope).await?;
        sqlx::query(sql(format!(
            "delete from {} where set_id = $1 and group_id = $2",
            self.table("group_set_group")
        )))
        .bind(set.as_uuid())
        .bind(group.as_uuid())
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// The group sets of a course with their groups; needs `courses.groups.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `not_found` for an unknown course.
    pub async fn group_sets(&self, actor: &Actor, course: CourseId) -> Result<Vec<GroupSet>> {
        let scope = self.course(course).await?.scope;
        self.access.require(actor, GROUPS_MANAGE, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select s.id, s.name, s.code, coalesce(array_agg(g.group_id order by g.group_id)
             filter (where g.group_id is not null), '{{}}') as groups
             from {} s left join {} g on g.set_id = s.id where s.course_id = $1
             group by s.id order by lower(s.name), s.id",
            self.table("group_set"),
            self.table("group_set_group")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| GroupSet {
                id: GroupSetId::from_uuid(r.get("id")),
                course,
                name: r.get("name"),
                code: r.get("code"),
                groups: r
                    .get::<Vec<uuid::Uuid>, _>("groups")
                    .into_iter()
                    .map(GroupId::from_uuid)
                    .collect(),
            })
            .collect())
    }

    /// Inserts a membership of a participant; one who is a member with that origin already stays.
    async fn insert_member(
        &self,
        group: &Group,
        course: &Course,
        account: PrincipalId,
        origin: &str,
    ) -> Result<()> {
        let not_participant = || {
            Error::validation(
                "courses.member_not_participant",
                vec![FieldError::new("account", "participant")],
            )
        };
        if !self.participants.takes_part(course.id, account).await? {
            return Err(not_participant());
        }
        if let Some(section) = group.section {
            let section = self.section(section).await?;
            if !self.sees(&Actor::Principal(account), section.scope).await? {
                return Err(not_participant());
            }
        }
        sqlx::query(sql(format!(
            "insert into {} (group_id, account_id, origin) values ($1, $2, $3) on conflict do nothing",
            self.table("group_member")
        )))
        .bind(group.id.as_uuid())
        .bind(account.as_uuid())
        .bind(origin)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.foreign_key_violation" {
                Error::validation(
                    "courses.member_unknown",
                    vec![FieldError::new("account", "account")],
                )
            } else {
                e
            }
        })
        .map(drop)
    }

    /// Deletes the membership of one origin.
    async fn delete_member(&self, group: GroupId, account: PrincipalId, origin: &str) -> Result<()> {
        sqlx::query(sql(format!(
            "delete from {} where group_id = $1 and account_id = $2 and origin = $3",
            self.table("group_member")
        )))
        .bind(group.as_uuid())
        .bind(account.as_uuid())
        .bind(origin)
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// A group and its course.
    async fn group_and_course(&self, id: GroupId) -> Result<(Group, Course)> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let group = self.read_group_in(&mut conn, id, false).await?;
        let course = self.read_course_in(&mut conn, group.course, false).await?;
        Ok((group, course))
    }

    /// The course of a group set.
    async fn set_course(&self, set: GroupSetId) -> Result<Course> {
        let course: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select course_id from {} where id = $1",
            self.table("group_set")
        )))
        .bind(set.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("courses.group_set_not_found"))?;
        self.course(CourseId::from_uuid(course)).await
    }

    /// Reads a group, locked for a change when `lock`.
    async fn read_group_in(&self, conn: &mut PgConnection, id: GroupId, lock: bool) -> Result<Group> {
        let lock = if lock { " for update" } else { "" };
        sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1{lock}",
            self.table("group")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .map(|row| group(&row))
        .ok_or_else(|| Error::not_found("courses.group_not_found"))
    }
}

impl Courses {
    /// The groups of a course a person belongs to, by any origin; for modules that resolve what applies to a
    /// person, which never show them to anyone.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn groups_of(&self, course: CourseId, account: PrincipalId) -> Result<Vec<GroupId>> {
        let rows: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct g.id from {} g join {} m on m.group_id = g.id
             where g.course_id = $1 and m.account_id = $2",
            self.table("group"),
            self.table("group_member")
        )))
        .bind(course.as_uuid())
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.into_iter().map(GroupId::from_uuid).collect())
    }
}
