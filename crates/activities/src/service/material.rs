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

//! Materials: the file of a course module of the `activities.file` type, owned through this module's own
//! link table and read by whoever sees the course module; and copying a course module with what it keeps.

use kampus_access::Actor;
use kampus_common::{Error, FieldError, Result};
use kampus_courses::{Place, SectionId};
use kampus_files::FileId;

use super::principal;
use crate::permissions::MODULE_MANAGE;
use crate::settings::files::MATERIAL_AREA;
use crate::settings::types::FILE;
use crate::{Activities, Module, ModuleFields, ModuleId, ModuleState, sql};

impl Activities {
    /// The file of a course module the actor sees, if it has one.
    ///
    /// # Errors
    ///
    /// `not_found` for a course module the actor does not see.
    pub async fn module_file(&self, actor: &Actor, id: ModuleId) -> Result<Option<FileId>> {
        self.module(actor, id).await?;
        self.file_of(id).await
    }

    /// The file of a course module, whoever asks.
    async fn file_of(&self, id: ModuleId) -> Result<Option<FileId>> {
        let file: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select file_id from {} where course_module_id = $1",
            self.table("module_file")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(file.map(FileId::from_uuid))
    }

    /// Makes a file the person uploaded into `activities.material` the file of a course module of the file
    /// type, replacing the one it had; needs `activities.module.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`activities.not_a_file_material`) for a course
    /// module of another type, and those of attaching a file.
    pub async fn set_module_file(&self, actor: &Actor, id: ModuleId, file: FileId) -> Result<FileId> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored_in(&mut tx, id, true).await?;
        self.access.require(actor, MODULE_MANAGE, stored.scope).await?;
        if stored.type_code != FILE {
            return Err(Error::validation(
                "activities.not_a_file_material",
                vec![FieldError::new("type", "file")],
            ));
        }
        if stored.state != ModuleState::Active {
            return Err(Error::conflict("activities.module_in_bin"));
        }
        let old: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select file_id from {} where course_module_id = $1 for update",
            self.table("module_file")
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.files
            .attach_in(&mut tx, file, MATERIAL_AREA, principal(actor))
            .await?;
        if let Some(old) = old {
            self.files.detach_in(&mut tx, FileId::from_uuid(old)).await?;
        }
        sqlx::query(sql(format!(
            "insert into {} (course_module_id, file_id) values ($1, $2)
             on conflict (course_module_id) do update set file_id = excluded.file_id",
            self.table("module_file")
        )))
        .bind(id.as_uuid())
        .bind(file.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set version = version + 1 where id = $1",
            self.table("course_module")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let updated = self.stored_in(&mut tx, id, false).await?;
        self.publish_in(&mut tx, crate::events::MODULE_UPDATED, &updated, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(file)
    }

    /// Whether an actor may read a file of a material: when they see the course module that owns it and meet
    /// its access conditions.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn file_readable(
        &self,
        actor: &Actor,
        file: uuid::Uuid,
        services: &kampus_module::ServiceContext,
    ) -> Result<bool> {
        let owner: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select course_module_id from {} where file_id = $1",
            self.table("module_file")
        )))
        .bind(file)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let Some(owner) = owner else { return Ok(false) };
        let module = self.stored(ModuleId::from_uuid(owner)).await?;
        if !self.visible_to(actor, &module).await? {
            return Ok(false);
        }
        Ok(self.availability(actor, &module, services).await?.available)
    }

    /// Copies a course module the actor sees to a place in a section, with its settings and, for a file
    /// material, its file as a file of its own; needs `activities.module.manage` in the section. The copy is
    /// one transaction.
    ///
    /// # Errors
    ///
    /// Those of adding a course module, and `not_found` for one the actor does not see.
    pub async fn copy_module(
        &self,
        actor: &Actor,
        id: ModuleId,
        section: SectionId,
        place: Place<ModuleId>,
    ) -> Result<Module> {
        let source = self.module(actor, id).await?;
        let target = self.courses.section(section).await?;
        self.access.require(actor, MODULE_MANAGE, target.scope).await?;
        let declared = self.enabled_type(&source.type_code).await?;
        let fields = ModuleFields {
            title: source.title.clone(),
            on_course_page: source.on_course_page,
            visibility: source.visibility,
            settings: source.settings.clone(),
        };
        let mut tx = self.db.begin().await?;
        let new = self
            .insert_module_in(&mut tx, actor, (&target, declared), fields, place)
            .await?;
        let created = self.stored_in(&mut tx, new, false).await?;
        if let Some(file) = self.file_of(id).await? {
            let copy = self.files.copy_attached_in(&mut tx, file, created.scope).await?;
            sqlx::query(sql(format!(
                "insert into {} (course_module_id, file_id) values ($1, $2)",
                self.table("module_file")
            )))
            .bind(new.as_uuid())
            .bind(copy.id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        self.publish_in(&mut tx, crate::events::MODULE_CREATED, &created, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }
}
