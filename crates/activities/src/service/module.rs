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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Course modules: added to a section in one transaction with the settings their type keeps, changed with
//! `If-Match`, moved between sections of their course, and deleted to the recycle bin of their course,
//! from where they are restored or removed for good when it expires.

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId, ScopeKind};
use kampus_common::{Error, ErrorKind, FieldError, Result, Timestamp};
use kampus_courses::support::sort_key;
use kampus_courses::{CourseId, Place, SectionId};
use kampus_db::references::ExternalReference;
use kampus_module::ActivityTypeDeclaration;
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};

use super::{module, principal};
use crate::permissions::MODULE_MANAGE;
use crate::{Activities, BinItem, Module, ModuleFields, ModuleId, ModuleState, sql};

/// The longest title, in characters.
const MAX_TITLE: usize = 255;

/// A trimmed title that is not empty or too long.
fn checked_title(title: &str, problems: &mut Vec<FieldError>) -> String {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > MAX_TITLE {
        problems.push(FieldError::new("title", "length"));
    }
    title.to_owned()
}

/// The settings of a course module: only the ones its type declares, each of its kind, with the defaults
/// of those left out.
fn checked_settings(
    declared: &ActivityTypeDeclaration,
    given: BTreeMap<String, Value>,
    problems: &mut Vec<FieldError>,
) -> BTreeMap<String, Value> {
    for (key, value) in &given {
        let fits = declared
            .settings
            .iter()
            .find(|s| s.key == key)
            .is_some_and(|s| s.kind.accepts(value));
        if !fits {
            problems.push(FieldError::new(format!("settings.{key}"), "type"));
        }
    }
    let mut settings = given;
    for setting in declared.settings {
        settings
            .entry(setting.key.to_owned())
            .or_insert_with(|| setting.kind.default_value());
    }
    settings
}

/// The error of a change made on another version than the stored one.
fn changed(stored: i32) -> Error {
    Error::new(ErrorKind::PreconditionFailed, "activities.module_changed")
        .with_param("version", stored.to_string())
}

impl Activities {
    /// A course module the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` (`activities.module_not_found`) for an unknown course module, one in the bin, or one the
    /// actor does not see; they answer the same.
    pub async fn module(&self, actor: &Actor, id: ModuleId) -> Result<Module> {
        let found = self.stored(id).await?;
        if self.visible_to(actor, &found).await? {
            Ok(found)
        } else {
            Err(Error::not_found("activities.module_not_found"))
        }
    }

    /// The course modules of a section the actor sees, in their order.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown section or one the actor does not see.
    pub async fn modules(&self, actor: &Actor, section: SectionId) -> Result<Vec<Module>> {
        let section = self.courses.section(section).await?;
        if !self.courses.sees(actor, section.scope).await? {
            return Err(Error::not_found("courses.section_not_found"));
        }
        let rows = sqlx::query(sql(format!(
            "{} where m.section_id = $1 and m.state = 'active' order by m.sort_key",
            self.select()
        )))
        .bind(section.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut visible = Vec::new();
        for row in &rows {
            let found = module(row);
            if self.visible_to(actor, &found).await? {
                visible.push(found);
            }
        }
        Ok(visible)
    }

    /// The siblings of a place in a section, in their order, in the bin or not, so a new key never meets one.
    async fn siblings_in(
        &self,
        conn: &mut PgConnection,
        section: SectionId,
        except: Option<ModuleId>,
    ) -> Result<Vec<(ModuleId, String)>> {
        let rows = sqlx::query(sql(format!(
            "select id, sort_key from {} where section_id = $1 order by sort_key",
            self.table("course_module")
        )))
        .bind(section.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (ModuleId::from_uuid(r.get("id")), r.get::<String, _>("sort_key")))
            .filter(|(id, _)| Some(*id) != except)
            .collect())
    }

    /// The declaration of an enabled type.
    pub(super) async fn enabled_type(&self, code: &str) -> Result<&'static ActivityTypeDeclaration> {
        let enabled: Option<bool> = sqlx::query_scalar(sql(format!(
            "select enabled from {} where code = $1 and retired_at is null",
            self.table("activity_type")
        )))
        .bind(code)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let declared = self.type_declaration(code);
        match (enabled, declared) {
            (Some(true), Some(declared)) => Ok(declared),
            (Some(false), _) => {
                Err(Error::conflict("activities.type_disabled").with_param("type", code.to_owned()))
            }
            _ => Err(
                Error::validation("activities.type_unknown", vec![FieldError::new("type", "one_of")])
                    .with_param("type", code.to_owned()),
            ),
        }
    }

    /// Adds a course module of a type to a section, at a place among its course modules; needs
    /// `activities.module.manage` in the section.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown section, `permission_denied` without the permission, `validation`
    /// (`activities.module_invalid`) for a title or settings that do not fit, (`activities.type_unknown`) for
    /// a type no module provides, `conflict` (`activities.type_disabled`) for one that is turned off.
    pub async fn create_module(
        &self,
        actor: &Actor,
        section: SectionId,
        type_code: &str,
        fields: ModuleFields,
        place: Place<ModuleId>,
    ) -> Result<Module> {
        self.create_module_with(actor, section, type_code, fields, place, None)
            .await
    }

    /// Adds a course module, with the identifier a system gives it when one is given, in one transaction.
    pub(crate) async fn create_module_with(
        &self,
        actor: &Actor,
        section: SectionId,
        type_code: &str,
        fields: ModuleFields,
        place: Place<ModuleId>,
        reference: Option<&ExternalReference>,
    ) -> Result<Module> {
        let section = self.courses.section(section).await?;
        self.access.require(actor, MODULE_MANAGE, section.scope).await?;
        let declared = self.enabled_type(type_code).await?;
        let mut tx = self.db.begin().await?;
        let id = self
            .insert_module_in(&mut tx, actor, (&section, declared), fields, place)
            .await?;
        if let Some(reference) = reference {
            let campus = self.courses.course(section.course).await?.campus;
            self.module_references()
                .set_in(&mut tx, campus.as_uuid(), id.as_uuid(), reference)
                .await?;
        }
        let created = self.stored_in(&mut tx, id, false).await?;
        self.publish_in(&mut tx, crate::events::MODULE_CREATED, &created, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// Inserts a course module with its scope and settings inside a transaction.
    pub(crate) async fn insert_module_in(
        &self,
        conn: &mut PgConnection,
        _actor: &Actor,
        (section, declared): (&kampus_courses::Section, &ActivityTypeDeclaration),
        fields: ModuleFields,
        place: Place<ModuleId>,
    ) -> Result<ModuleId> {
        let mut problems = Vec::new();
        let title = checked_title(&fields.title, &mut problems);
        let settings = checked_settings(declared, fields.settings, &mut problems);
        let visibility = fields.visibility.validate()?;
        if !problems.is_empty() {
            return Err(Error::validation("activities.module_invalid", problems));
        }
        let siblings = self.siblings_in(conn, section.id, None).await?;
        let key = sort_key::place(&siblings, place)?;
        let scope = self
            .access
            .create_scope_in(conn, section.scope, ScopeKind::Activity)
            .await?;
        let (from, until) = visibility.window();
        let id = ModuleId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, scope_id, section_id, type_code, title, sort_key, visibility, visible_from,
             visible_until, on_course_page) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            self.table("course_module")
        )))
        .bind(id.as_uuid())
        .bind(scope.as_uuid())
        .bind(section.id.as_uuid())
        .bind(declared.code)
        .bind(&title)
        .bind(&key)
        .bind(visibility.mode())
        .bind(from.map(|f| f.as_offset_date_time()))
        .bind(until.map(|u| u.as_offset_date_time()))
        .bind(fields.on_course_page)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        self.write_settings_in(conn, id, &settings).await?;
        Ok(id)
    }

    /// Stores the settings of a course module.
    async fn write_settings_in(
        &self,
        conn: &mut PgConnection,
        id: ModuleId,
        settings: &BTreeMap<String, Value>,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (course_module_id, values) values ($1, $2)
             on conflict (course_module_id) do update set values = excluded.values",
            self.table("module_setting")
        )))
        .bind(id.as_uuid())
        .bind(json!(settings))
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    }

    /// Replaces the title, visibility, listing and settings of a course module, on the version it was read
    /// at; needs `activities.module.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` as for adding, and `precondition_failed`
    /// (`activities.module_changed`) when it changed since it was read.
    pub async fn update_module(
        &self,
        actor: &Actor,
        id: ModuleId,
        fields: ModuleFields,
        expected_version: Option<i32>,
    ) -> Result<Module> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored_in(&mut tx, id, true).await?;
        self.access.require(actor, MODULE_MANAGE, stored.scope).await?;
        if stored.state != ModuleState::Active {
            return Err(Error::conflict("activities.module_in_bin"));
        }
        if expected_version.is_some_and(|v| v != stored.version) {
            return Err(changed(stored.version));
        }
        let declared = self
            .type_declaration(&stored.type_code)
            .ok_or_else(|| Error::conflict("activities.type_unknown"))?;
        let mut problems = Vec::new();
        let title = checked_title(&fields.title, &mut problems);
        let settings = checked_settings(declared, fields.settings, &mut problems);
        let visibility = fields.visibility.validate()?;
        if !problems.is_empty() {
            return Err(Error::validation("activities.module_invalid", problems));
        }
        let (from, until) = visibility.window();
        sqlx::query(sql(format!(
            "update {} set title = $2, visibility = $3, visible_from = $4, visible_until = $5, on_course_page = $6,
             version = version + 1 where id = $1",
            self.table("course_module")
        )))
        .bind(id.as_uuid())
        .bind(&title)
        .bind(visibility.mode())
        .bind(from.map(|f| f.as_offset_date_time()))
        .bind(until.map(|u| u.as_offset_date_time()))
        .bind(fields.on_course_page)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.write_settings_in(&mut tx, id, &settings).await?;
        let updated = self.stored_in(&mut tx, id, false).await?;
        self.publish_in(&mut tx, crate::events::MODULE_UPDATED, &updated, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(updated)
    }

    /// Moves a course module to a place among the course modules of a section of its course, taking its
    /// scope with it; needs `activities.module.manage` in it and in the section.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`activities.section_of_another_course`) for a
    /// section of another course, (`courses.sibling_unknown`) for a sibling that is not among them.
    pub async fn move_module(
        &self,
        actor: &Actor,
        id: ModuleId,
        section: SectionId,
        place: Place<ModuleId>,
    ) -> Result<Module> {
        let target = self.courses.section(section).await?;
        let mut tx = self.db.begin().await?;
        let stored = self.stored_in(&mut tx, id, true).await?;
        self.access.require(actor, MODULE_MANAGE, stored.scope).await?;
        self.access.require(actor, MODULE_MANAGE, target.scope).await?;
        if stored.state != ModuleState::Active {
            return Err(Error::conflict("activities.module_in_bin"));
        }
        let current = self.courses.section(stored.section).await?;
        if current.course != target.course {
            return Err(Error::validation(
                "activities.section_of_another_course",
                vec![FieldError::new("section", "course")],
            ));
        }
        let siblings = self.siblings_in(&mut tx, target.id, Some(id)).await?;
        let key = sort_key::place(&siblings, place)?;
        if target.id != stored.section {
            self.access
                .move_scope_in(&mut tx, stored.scope, target.scope)
                .await?;
        }
        sqlx::query(sql(format!(
            "update {} set section_id = $2, sort_key = $3, version = version + 1 where id = $1",
            self.table("course_module")
        )))
        .bind(id.as_uuid())
        .bind(target.id.as_uuid())
        .bind(&key)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let moved = self.stored_in(&mut tx, id, false).await?;
        self.publish_in(&mut tx, crate::events::MODULE_MOVED, &moved, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(moved)
    }

    /// The course of a section.
    async fn course_of(&self, section: SectionId) -> Result<CourseId> {
        Ok(self.courses.section(section).await?.course)
    }

    /// Deletes a course module to the recycle bin of its course, where it stays 30 days; needs
    /// `activities.module.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `conflict` (`activities.module_in_bin`) when it is already there.
    pub async fn delete_module(&self, actor: &Actor, id: ModuleId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored_in(&mut tx, id, true).await?;
        self.access.require(actor, MODULE_MANAGE, stored.scope).await?;
        if stored.state == ModuleState::InBin {
            return Err(Error::conflict("activities.module_in_bin"));
        }
        let course = self.course_of(stored.section).await?;
        sqlx::query(sql(format!(
            "update {} set state = 'in_bin', version = version + 1 where id = $1",
            self.table("course_module")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {} (id, course_id, course_module_id, deleted_by, expires_at)
             values ($1, $2, $3, $4, now() + interval '30 days')",
            self.table("bin_item")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(course.as_uuid())
        .bind(id.as_uuid())
        .bind(principal(actor))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(&mut tx, crate::events::MODULE_DELETED, &stored, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Brings a course module back from the recycle bin to its section and place; needs
    /// `activities.module.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `conflict` (`activities.module_not_in_bin`) for one that is not in the
    /// bin, (`activities.section_gone`) when its section was deleted meanwhile.
    pub async fn restore_module(&self, actor: &Actor, id: ModuleId) -> Result<Module> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored_in(&mut tx, id, true).await?;
        self.access.require(actor, MODULE_MANAGE, stored.scope).await?;
        if stored.state != ModuleState::InBin {
            return Err(Error::conflict("activities.module_not_in_bin"));
        }
        self.courses
            .section(stored.section)
            .await
            .map_err(|_| Error::conflict("activities.section_gone"))?;
        sqlx::query(sql(format!(
            "update {} set state = 'active', version = version + 1 where id = $1",
            self.table("course_module")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "delete from {} where course_module_id = $1",
            self.table("bin_item")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let restored = self.stored_in(&mut tx, id, false).await?;
        self.publish_in(
            &mut tx,
            crate::events::MODULE_RESTORED,
            &restored,
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(restored)
    }

    /// What is in the recycle bin of a course; needs `activities.module.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course, `permission_denied` without the permission.
    pub async fn bin(&self, actor: &Actor, course: CourseId) -> Result<Vec<BinItem>> {
        let course = self.courses.course(course).await?;
        self.access.require(actor, MODULE_MANAGE, course.scope).await?;
        let rows = sqlx::query(sql(format!(
            "select m.id, m.title, m.type_code, b.deleted_at, b.expires_at, b.error from {} b
             join {} m on m.id = b.course_module_id where b.course_id = $1 order by b.deleted_at desc",
            self.table("bin_item"),
            self.table("course_module")
        )))
        .bind(course.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| BinItem {
                module: ModuleId::from_uuid(r.get("id")),
                title: r.get("title"),
                type_code: r.get("type_code"),
                deleted_at: Timestamp::from_offset_date_time(r.get("deleted_at")),
                expires_at: Timestamp::from_offset_date_time(r.get("expires_at")),
                error: r.get("error"),
            })
            .collect())
    }

    /// The type of a course module, whoever asks, in the bin or not.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn type_of(&self, id: ModuleId) -> Result<String> {
        Ok(self.stored(id).await?.type_code)
    }

    /// The scope of a course module, for checks of modules that add to it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn scope_of(&self, id: ModuleId) -> Result<ScopeId> {
        Ok(self.stored(id).await?.scope)
    }

    /// The course a course module belongs to.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn course_of_module(&self, id: ModuleId) -> Result<kampus_courses::CourseId> {
        let module = self.stored(id).await?;
        Ok(self.courses.section(module.section).await?.course)
    }
}

impl Activities {
    /// The scope of a section, for the permission to add to it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown or deleted section.
    pub async fn section_scope(&self, section: SectionId) -> Result<ScopeId> {
        Ok(self.courses.section(section).await?.scope)
    }
}
