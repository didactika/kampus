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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Publishing: an event is written with its outbox entry in the transaction of the change it reports, so
//! a change that rolls back publishes nothing; an audited event writes its audit entry in the same
//! transaction.

use kampus_common::{Error, FieldError, Result};
use kampus_module::Event;
use serde_json::Value;
use sqlx::{PgConnection, Row};

use crate::{Events, sql};

/// How long a chain of events published by listeners may grow.
pub const MAX_DEPTH: u8 = 10;

/// An event to publish.
#[derive(Debug, Clone, PartialEq)]
pub struct Publish<'a> {
    /// Its declared type.
    pub code: &'a str,
    /// The scope it happened in.
    pub scope: Option<uuid::Uuid>,
    /// The campus of that scope, for the audit log.
    pub campus: Option<uuid::Uuid>,
    /// Who made it happen.
    pub actor: Option<uuid::Uuid>,
    /// What it is about.
    pub object: Option<uuid::Uuid>,
    /// What happened: an object with every field its type declares.
    pub payload: Value,
    /// What the audit log says about it, for an audited type; its code when absent.
    pub summary: Option<String>,
    /// The event whose listener publishes this one, so the chain is measured.
    pub cause: Option<&'a Event>,
}

impl<'a> Publish<'a> {
    /// An event of a type about an object, with its payload.
    #[must_use]
    pub const fn new(code: &'a str, object: Option<uuid::Uuid>, payload: Value) -> Self {
        Self {
            code,
            scope: None,
            campus: None,
            actor: None,
            object,
            payload,
            summary: None,
            cause: None,
        }
    }
}

impl Events {
    /// Publishes an event inside the caller's transaction; listeners receive it after the transaction
    /// commits, and never if it rolls back.
    ///
    /// # Errors
    ///
    /// `validation` (`events.type_unknown`) for an undeclared type or `events.any`, `validation` (`events.payload_invalid`)
    /// naming the declared fields the payload lacks, `conflict` (`events.chain_limit_reached`) when
    /// listeners published more than [`MAX_DEPTH`] events in a chain.
    pub async fn publish_in(&self, conn: &mut PgConnection, event: Publish<'_>) -> Result<uuid::Uuid> {
        if event.code == crate::ANY {
            return Err(
                Error::validation("events.type_unknown", vec![FieldError::new("code", "declared")])
                    .with_param("code", event.code.to_owned()),
            );
        }
        let declared = sqlx::query(sql(format!(
            "select version, fields, audited from {} where code = $1 and retired_at is null",
            self.table("event_type")
        )))
        .bind(event.code)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| {
            Error::validation("events.type_unknown", vec![FieldError::new("code", "declared")])
                .with_param("code", event.code.to_owned())
        })?;
        let fields: Vec<String> =
            serde_json::from_value(declared.get::<Value, _>("fields")).unwrap_or_default();
        let missing: Vec<FieldError> = fields
            .iter()
            .filter(|f| event.payload.get(f.as_str()).is_none())
            .map(|f| FieldError::new(format!("payload.{f}"), "required"))
            .collect();
        if !event.payload.is_object() || !missing.is_empty() {
            return Err(Error::validation("events.payload_invalid", missing)
                .with_param("code", event.code.to_owned()));
        }
        let depth = event.cause.map_or(0, |c| c.depth.saturating_add(1));
        if depth > MAX_DEPTH {
            tracing::error!(
                code = event.code,
                depth,
                "a chain of events reached its limit and stopped"
            );
            return Err(
                Error::conflict("events.chain_limit_reached").with_param("code", event.code.to_owned())
            );
        }
        let id = uuid::Uuid::now_v7();
        let request = kampus_module::http::REQUEST_ID.try_with(Clone::clone).ok();
        sqlx::query(sql(format!(
            "insert into {} (id, type_code, type_version, scope, actor, object, payload, depth, request)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            self.table("event")
        )))
        .bind(id)
        .bind(event.code)
        .bind(declared.get::<i32, _>("version"))
        .bind(event.scope)
        .bind(event.actor)
        .bind(event.object)
        .bind(&event.payload)
        .bind(i32::from(depth))
        .bind(request)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {} (event_id) values ($1)",
            self.table("outbox")
        )))
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        if declared.get::<bool, _>("audited") {
            sqlx::query(sql(format!(
                "insert into {} (id, event_id, campus, scope, actor, object, summary)
                 values ($1, $2, $3, $4, $5, $6, $7)",
                self.table("audit_entry")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(id)
            .bind(event.campus)
            .bind(event.scope)
            .bind(event.actor)
            .bind(event.object)
            .bind(event.summary.as_deref().unwrap_or(event.code))
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(id)
    }
}
