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

//! The service API of activities, one file per area, and what they share: reading course modules from
//! their rows, who sees one, and the types modules provide.

pub mod exception;
pub mod material;
pub mod module;
pub mod purge;
pub mod reference;

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::{SectionId, Visibility};
use kampus_module::Module as Declared;
use serde_json::{Value, json};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use crate::{Activities, ActivityType, Module, ModuleId, ModuleState, sql};

/// The columns of a course module with its settings.
pub(crate) const COLUMNS: &str =
    "m.id, m.scope_id, m.section_id, m.type_code, m.title, m.sort_key, m.visibility,
    m.visible_from, m.visible_until, m.on_course_page, m.state, m.version,
    coalesce(s.values, '{}'::jsonb) as settings";

/// An instant read from a nullable column.
fn instant(row: &PgRow, column: &str) -> Option<Timestamp> {
    row.get::<Option<time::OffsetDateTime>, _>(column)
        .map(Timestamp::from_offset_date_time)
}

/// A course module from its row.
pub(crate) fn module(row: &PgRow) -> Module {
    let state = if row.get::<String, _>("state") == "in_bin" {
        ModuleState::InBin
    } else {
        ModuleState::Active
    };
    let mode: String = row.get("visibility");
    Module {
        id: ModuleId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        section: SectionId::from_uuid(row.get("section_id")),
        type_code: row.get("type_code"),
        title: row.get("title"),
        sort_key: row.get("sort_key"),
        visibility: Visibility::read(&mode, instant(row, "visible_from"), instant(row, "visible_until")),
        on_course_page: row.get("on_course_page"),
        settings: serde_json::from_value::<BTreeMap<String, Value>>(row.get("settings")).unwrap_or_default(),
        state,
        version: row.get("version"),
    }
}

impl Activities {
    /// The select of course modules with their settings.
    pub(crate) fn select(&self) -> String {
        format!(
            "select {COLUMNS} from {} m left join {} s on s.course_module_id = m.id",
            self.table("course_module"),
            self.table("module_setting")
        )
    }

    /// A course module in any state, whoever asks.
    pub(crate) async fn stored_in(
        &self,
        conn: &mut PgConnection,
        id: ModuleId,
        lock: bool,
    ) -> Result<Module> {
        let row = sqlx::query(sql(format!(
            "{} where m.id = $1 {}",
            self.select(),
            if lock { "for update of m" } else { "" }
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("activities.module_not_found"))?;
        Ok(module(&row))
    }

    /// A course module in any state, whoever asks.
    pub(crate) async fn stored(&self, id: ModuleId) -> Result<Module> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.stored_in(&mut conn, id, false).await
    }

    /// Whether an actor sees a course module: it is in its section, its section and the nodes above are
    /// shown to them, and it is shown, or they hold `courses.hidden.view` in it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown section.
    pub async fn visible_to(&self, actor: &Actor, module: &Module) -> Result<bool> {
        if module.state != ModuleState::Active {
            return Ok(false);
        }
        let section = self.courses.section(module.section).await?;
        if !self.courses.sees(actor, section.scope).await? {
            return Ok(false);
        }
        if module
            .visibility
            .shown_at(Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc()))
        {
            return Ok(true);
        }
        Ok(self
            .access
            .check(actor, kampus_courses::permissions::HIDDEN_VIEW, module.scope)
            .await?
            .allowed)
    }

    /// Brings the types the modules of the build provide into the database; those no module provides any
    /// more are retired, and their course modules stay.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sync_declarations(&self, modules: &[&Declared]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut codes = Vec::new();
        for module in modules {
            for declared in module.activity_types {
                codes.push(declared.code.to_owned());
                let settings: Vec<Value> = declared
                    .settings
                    .iter()
                    .map(|s| json!({ "key": s.key, "kind": s.kind.stored(), "default": s.kind.default_value() }))
                    .collect();
                sqlx::query(sql(format!(
                    "insert into {} (code, module, kind, summary, features, settings) values ($1, $2, $3, $4, $5, $6)
                     on conflict (code) do update set module = excluded.module, kind = excluded.kind,
                     summary = excluded.summary, features = excluded.features, settings = excluded.settings,
                     retired_at = null",
                    self.table("activity_type")
                )))
                .bind(declared.code)
                .bind(module.name)
                .bind(declared.kind.as_str())
                .bind(declared.summary)
                .bind(json!(declared.features))
                .bind(json!(settings))
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where not (code = any($1)) and retired_at is null",
            self.table("activity_type")
        )))
        .bind(&codes)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The types course modules may be added of.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn types(&self) -> Result<Vec<ActivityType>> {
        let rows = sqlx::query(sql(format!(
            "select code, kind, summary, features, enabled from {} where retired_at is null order by code",
            self.table("activity_type")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| ActivityType {
                code: r.get("code"),
                kind: r.get("kind"),
                summary: r.get("summary"),
                features: serde_json::from_value(r.get("features")).unwrap_or_default(),
                enabled: r.get("enabled"),
            })
            .collect())
    }

    /// Publishes an event about a course module, in the transaction of the change.
    pub(crate) async fn publish_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        module: &Module,
        actor: Option<uuid::Uuid>,
    ) -> Result<()> {
        let mut event = kampus_events::Publish::new(
            code,
            Some(module.id.as_uuid()),
            json!({ "module": module.id, "section": module.section, "type": module.type_code }),
        );
        event.scope = Some(module.scope.as_uuid());
        event.actor = actor;
        event.summary = Some(format!("{code} {}", module.id));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }
}

/// The principal behind an actor.
pub(crate) const fn principal(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(p.as_uuid()),
        Actor::Visitor(_) => None,
    }
}
