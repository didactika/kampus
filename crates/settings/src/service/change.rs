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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Changes: a value is set only at a level its definition allows, by someone holding its permission
//! there, never below a scope that locks it, and only when it is of the declared kind. Every change keeps
//! who made it with the value before and after, and publishes `settings.changed`.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};

use crate::{SettingChange, Settings, sql};

/// Who writes a value.
#[derive(Debug, Clone, Copy)]
enum Writer<'a> {
    /// A person or a token, with the permission of the definition.
    Actor(&'a Actor),
    /// The operator, on the command line.
    Operator,
}

/// A value to write: absent to remove it.
#[derive(Debug, Clone, PartialEq)]
struct Write<'a> {
    /// The key.
    key: &'a str,
    /// The scope.
    scope: ScopeId,
    /// The value; removed when absent.
    value: Option<Value>,
    /// Whether it locks the scopes below.
    locked: bool,
    /// The version read, when it was read.
    expected_version: Option<i32>,
}

impl Settings {
    /// Sets a setting in a scope, locked or not; needs the permission of its definition there.
    /// `expected_version` is the version read, when the value was read before.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown key, `validation` (`settings.value_invalid`) for a value of another kind
    /// or out of its bounds, `validation` (`settings.level_invalid`) for a scope its definition does not
    /// allow, `permission_denied` without the permission, `conflict` (`settings.locked_above`) below a
    /// scope that locks it, `precondition_failed` when it changed since it was read.
    pub async fn set(
        &self,
        actor: &Actor,
        key: &str,
        scope: ScopeId,
        value: Value,
        locked: bool,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let write = Write {
            key,
            scope,
            value: Some(value),
            locked,
            expected_version,
        };
        self.write(Writer::Actor(actor), write).await
    }

    /// Removes the value of a setting in a scope, so it comes from above again.
    ///
    /// # Errors
    ///
    /// As [`Settings::set`].
    pub async fn unset(&self, actor: &Actor, key: &str, scope: ScopeId) -> Result<()> {
        let write = Write {
            key,
            scope,
            value: None,
            locked: false,
            expected_version: None,
        };
        self.write(Writer::Actor(actor), write).await
    }

    /// Sets or removes a value as the operator, on the command line: the checks of the definition apply,
    /// but no permission is needed.
    ///
    /// # Errors
    ///
    /// As [`Settings::set`], without `permission_denied`.
    pub async fn set_as_operator(
        &self,
        key: &str,
        scope: ScopeId,
        value: Option<Value>,
        locked: bool,
    ) -> Result<()> {
        let write = Write {
            key,
            scope,
            value,
            locked,
            expected_version: None,
        };
        self.write(Writer::Operator, write).await
    }

    /// The changes of a setting in a scope, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn history(&self, key: &str, scope: ScopeId) -> Result<Vec<SettingChange>> {
        let rows = sqlx::query(sql(format!(
            "select definition_key, scope_id, actor, before, after, locked, at from {}
             where definition_key = $1 and scope_id = $2 order by at desc, id desc",
            self.table("change")
        )))
        .bind(key)
        .bind(scope.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| SettingChange {
                key: r.get("definition_key"),
                scope: ScopeId::from_uuid(r.get("scope_id")),
                actor: r.get("actor"),
                before: r.get("before"),
                after: r.get("after"),
                locked: r.get("locked"),
                at: Timestamp::from_offset_date_time(r.get("at")),
            })
            .collect())
    }

    /// Writes or removes a value after its checks, records the change and publishes it.
    async fn write(&self, writer: Writer<'_>, write: Write<'_>) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let actor = self.check_in(&mut tx, writer, &write).await?;
        let before = self.stored_in(&mut tx, write.key, write.scope).await?;
        if let (Some(expected), Some((_, version))) = (write.expected_version, &before)
            && expected != *version
        {
            return Err(Error::new(
                kampus_common::ErrorKind::PreconditionFailed,
                "settings.changed_meanwhile",
            )
            .with_param("version", version.to_string()));
        }
        self.store_in(&mut tx, &write).await?;
        self.record_in(&mut tx, &write, actor, before.map(|(v, _)| v))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Checks a write against its definition, the permission and the locks above; returns the actor's
    /// principal.
    async fn check_in(
        &self,
        tx: &mut PgConnection,
        writer: Writer<'_>,
        write: &Write<'_>,
    ) -> Result<Option<uuid::Uuid>> {
        let definition = sqlx::query(sql(format!(
            "select kind, levels, permission from {} where key = $1 and retired_at is null",
            self.table("definition")
        )))
        .bind(write.key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("settings.unknown").with_param("key", write.key.to_owned()))?;
        if let Some(value) = &write.value
            && !super::accepts(&definition.get::<Value, _>("kind"), value)
        {
            return Err(Error::validation(
                "settings.value_invalid",
                vec![FieldError::new("value", "kind")],
            ));
        }
        let path = self.access.path(&mut *tx, write.scope).await?;
        let levels: Vec<String> = serde_json::from_value(definition.get("levels")).unwrap_or_default();
        if !levels.iter().any(|l| l == path[0].kind.as_str()) {
            return Err(
                Error::validation("settings.level_invalid", vec![FieldError::new("scope", "level")])
                    .with_param("allowed", levels.join(", ")),
            );
        }
        let actor = match writer {
            Writer::Actor(actor) => {
                self.access
                    .require(actor, &definition.get::<String, _>("permission"), write.scope)
                    .await?;
                match actor {
                    Actor::Principal(p) | Actor::ViewAs(p, _) => Some(p.as_uuid()),
                    Actor::Visitor(_) => None,
                }
            }
            Writer::Operator => None,
        };
        let above: Vec<uuid::Uuid> = path.iter().skip(1).map(|s| s.id.as_uuid()).collect();
        let locked_above: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where definition_key = $1 and scope_id = any($2) and locked)",
            self.table("value")
        )))
        .bind(write.key)
        .bind(&above)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if locked_above {
            return Err(Error::conflict("settings.locked_above").with_param("key", write.key.to_owned()));
        }
        Ok(actor)
    }

    /// Stores or removes the value.
    async fn store_in(&self, tx: &mut PgConnection, write: &Write<'_>) -> Result<()> {
        let values = self.table("value");
        let query = match &write.value {
            Some(value) => sqlx::query(sql(format!(
                "insert into {values} (id, definition_key, scope_id, value, locked) values ($1, $2, $3, $4, $5)
                 on conflict (definition_key, scope_id) do update set value = excluded.value,
                 locked = excluded.locked, version = {values}.version + 1, updated_at = now()"
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(write.key)
            .bind(write.scope.as_uuid())
            .bind(value)
            .bind(write.locked),
            None => sqlx::query(sql(format!(
                "delete from {values} where definition_key = $1 and scope_id = $2"
            )))
            .bind(write.key)
            .bind(write.scope.as_uuid()),
        };
        query.execute(tx).await.map(drop).map_err(kampus_db::classify)
    }

    /// Records a change and publishes it, in the transaction of the change.
    async fn record_in(
        &self,
        tx: &mut PgConnection,
        write: &Write<'_>,
        actor: Option<uuid::Uuid>,
        before: Option<Value>,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, definition_key, scope_id, actor, before, after, locked) values ($1, $2, $3, $4, $5, $6, $7)",
            self.table("change")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(write.key)
        .bind(write.scope.as_uuid())
        .bind(actor)
        .bind(before)
        .bind(&write.value)
        .bind(write.locked && write.value.is_some())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut event = kampus_events::Publish::new(
            crate::events::CHANGED,
            Some(write.scope.as_uuid()),
            json!({ "key": write.key, "scope": write.scope }),
        );
        event.scope = Some(write.scope.as_uuid());
        event.actor = actor;
        event.summary = Some(format!("setting {} changed", write.key));
        kampus_events::Events::new(self.db.clone())
            .publish_in(tx, event)
            .await
            .map(drop)
    }

    /// The value set in a scope with its version, if any.
    async fn stored_in(
        &self,
        conn: &mut PgConnection,
        key: &str,
        scope: ScopeId,
    ) -> Result<Option<(Value, i32)>> {
        let row = sqlx::query(sql(format!(
            "select value, version from {} where definition_key = $1 and scope_id = $2 for update",
            self.table("value")
        )))
        .bind(key)
        .bind(scope.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.map(|r| (r.get("value"), r.get("version"))))
    }
}
