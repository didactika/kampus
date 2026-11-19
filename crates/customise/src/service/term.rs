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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Terminology: the words of each module's domain, which a scope renames per language; the word of the
//! nearest scope that renames it applies, unless a scope above locks its own.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde_json::{Value, json};
use sqlx::Row;

use super::{SetIn, choose, principal};
use crate::permissions::TERM_MANAGE;
use crate::{Customise, Origin, Term, sql};

/// The longest form of a term, in characters.
const MAX_FORM: usize = 100;

/// The declared forms of a term in a language, or in its first language.
fn declared(forms: &Value, language: &str) -> Option<(String, String)> {
    let all = forms.as_array()?;
    let chosen = all
        .iter()
        .find(|f| f.get("language").and_then(Value::as_str) == Some(language))
        .or_else(|| all.first())?;
    Some((
        chosen.get("singular")?.as_str()?.to_owned(),
        chosen.get("plural")?.as_str()?.to_owned(),
    ))
}

impl Customise {
    /// The word for a term in a scope and a language, with where it came from.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.term_unknown`) for a term no module declares, `not_found` for an unknown
    /// scope.
    pub async fn term(&self, code: &str, scope: ScopeId, language: &str) -> Result<Term> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let forms: Value = sqlx::query_scalar(sql(format!(
            "select forms from {} where code = $1 and retired_at is null",
            self.table("term")
        )))
        .bind(code)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("customise.term_unknown").with_param("code", code.to_owned()))?;
        let depths = self.depths(&mut conn, scope).await?;
        let ids: Vec<uuid::Uuid> = depths.keys().copied().collect();
        let rows = sqlx::query(sql(format!(
            "select scope_id, singular, plural, locked from {} where term_code = $1 and language_code = $2
             and scope_id = any($3)",
            self.table("term_override")
        )))
        .bind(code)
        .bind(language)
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let set: Vec<SetIn<(String, String)>> = rows
            .iter()
            .map(|r| {
                let id: uuid::Uuid = r.get("scope_id");
                (
                    depths.get(&id).copied().unwrap_or(i32::MAX),
                    ScopeId::from_uuid(id),
                    (r.get("singular"), r.get("plural")),
                    r.get("locked"),
                )
            })
            .collect();
        let ((singular, plural), origin) = match choose(set) {
            Some(found) => found,
            None => (
                declared(&forms, language).ok_or_else(|| Error::internal("customise.term_without_forms"))?,
                Origin::Declared,
            ),
        };
        Ok(Term {
            code: code.to_owned(),
            language: language.to_owned(),
            singular,
            plural,
            origin,
        })
    }

    /// Renames a term in a scope and a language, locked or not; needs `customise.term.manage` there.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown term, `validation` (`customise.term_invalid`) for empty or long forms,
    /// `permission_denied` without the permission, `conflict` (`customise.locked_above`) below a scope
    /// that locks it.
    pub async fn rename_term(
        &self,
        actor: &Actor,
        code: &str,
        scope: ScopeId,
        language: &str,
        (singular, plural): (&str, &str),
        locked: bool,
    ) -> Result<Term> {
        let (singular, plural) = (singular.trim(), plural.trim());
        let mut problems = Vec::new();
        for (name, form) in [("singular", singular), ("plural", plural)] {
            if form.is_empty() || form.chars().count() > MAX_FORM {
                problems.push(FieldError::new(name, "length"));
            }
        }
        if !problems.is_empty() {
            return Err(Error::validation("customise.term_invalid", problems));
        }
        self.access.require(actor, TERM_MANAGE, scope).await?;
        let current = self.term(code, scope, language).await?;
        if let Origin::Scope {
            scope: at,
            locked: true,
        } = current.origin
            && at != scope
        {
            return Err(Error::conflict("customise.locked_above"));
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (term_code, scope_id, language_code, singular, plural, locked) values ($1, $2, $3, $4, $5, $6)
             on conflict (term_code, scope_id, language_code) do update set singular = excluded.singular,
             plural = excluded.plural, locked = excluded.locked, version = {}.version + 1",
            self.table("term_override"),
            self.table("term_override")
        )))
        .bind(code)
        .bind(scope.as_uuid())
        .bind(language)
        .bind(singular)
        .bind(plural)
        .bind(locked)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.record_in(
            &mut tx,
            "term",
            &format!("{code}/{language}"),
            scope,
            principal(actor),
            (
                Some(json!({ "singular": current.singular, "plural": current.plural })),
                Some(json!({ "singular": singular, "plural": plural, "locked": locked })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.term(code, scope, language).await
    }

    /// Removes the word a scope set for a term, so it comes from above again.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `customise.term.manage`.
    pub async fn reset_term(
        &self,
        actor: &Actor,
        code: &str,
        scope: ScopeId,
        language: &str,
    ) -> Result<Term> {
        self.access.require(actor, TERM_MANAGE, scope).await?;
        let mut tx = self.db.begin().await?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where term_code = $1 and scope_id = $2 and language_code = $3",
            self.table("term_override")
        )))
        .bind(code)
        .bind(scope.as_uuid())
        .bind(language)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed > 0 {
            self.record_in(
                &mut tx,
                "term",
                &format!("{code}/{language}"),
                scope,
                principal(actor),
                (None, None),
            )
            .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.term(code, scope, language).await
    }
}
