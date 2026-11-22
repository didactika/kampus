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

//! Themes: design tokens a scope sets over those of the scopes above it, down to where a scope locks its
//! own. Every token keeps to its kind, and a theme whose text colours lose WCAG 2.2 AA contrast on their
//! backgrounds is refused.

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde_json::{Value, json};
use sqlx::Row;

use super::principal;
use crate::model::colour::Rgb;
use crate::model::theme::{TOKENS, TokenKind};
use crate::permissions::THEME_MANAGE;
use crate::{Customise, Origin, Theme, sql};

/// The longest list of font families, in characters.
const MAX_FONT_FAMILY: usize = 200;

/// The tokens a scope sets.
pub(super) type Tokens = BTreeMap<String, Value>;

/// The value of a token that fits its kind, normalised; none when it does not fit.
fn fitted(kind: TokenKind, value: &Value) -> Option<Value> {
    match (kind, value) {
        (TokenKind::Colour, Value::String(text)) => {
            Rgb::parse(text).map(|_| json!(text.to_ascii_lowercase()))
        }
        (TokenKind::Pixels { min, max }, Value::Number(n)) => {
            n.as_i64().filter(|n| (min..=max).contains(n)).map(|n| json!(n))
        }
        (TokenKind::FontFamily, Value::String(text)) => {
            let fits = !text.trim().is_empty()
                && text.chars().count() <= MAX_FONT_FAMILY
                && text.chars().all(|c| c.is_alphanumeric() || " ,-'\"".contains(c));
            fits.then(|| json!(text.trim()))
        }
        (TokenKind::Choice(options), Value::String(text)) => {
            options.contains(&text.as_str()).then(|| json!(text))
        }
        _ => None,
    }
}

/// The tokens of a theme that fit their kinds, normalised, or the problems of those that do not.
pub(super) fn checked(tokens: &Tokens) -> std::result::Result<Tokens, Vec<FieldError>> {
    let mut problems = Vec::new();
    let mut fitting = Tokens::new();
    for (name, value) in tokens {
        match TOKENS
            .iter()
            .find(|t| t.name == name)
            .and_then(|t| fitted(t.kind, value))
        {
            Some(value) => {
                fitting.insert(name.clone(), value);
            }
            None => problems.push(FieldError::new(format!("tokens.{name}"), "type")),
        }
    }
    if problems.is_empty() {
        Ok(fitting)
    } else {
        Err(problems)
    }
}

/// The colour pairs of a theme, in either scheme, whose contrast is too low. Roles are grown to keep it, so
/// this is a last guard rather than a rule a seed can break.
pub(super) fn low_contrast(theme: &Theme) -> Vec<FieldError> {
    [("light", &theme.light), ("dark", &theme.dark)]
        .into_iter()
        .flat_map(|(mode, scheme)| {
            scheme.shortfalls().into_iter().map(move |(text, background, _)| {
                FieldError::new(format!("tokens.{text}"), format!("contrast_{mode}_{background}"))
            })
        })
        .collect()
}

/// The theme every scope starts from: the default of every token.
fn defaults() -> (Tokens, BTreeMap<String, Origin>) {
    let mut tokens = Tokens::new();
    let mut origins = BTreeMap::new();
    for token in TOKENS {
        let value = match token.kind {
            TokenKind::Pixels { .. } => token.default.parse::<i64>().map_or(Value::Null, |n| json!(n)),
            _ => json!(token.default),
        };
        tokens.insert(token.name.to_owned(), value);
        origins.insert(token.name.to_owned(), Origin::Declared);
    }
    (tokens, origins)
}

/// The themes set along a path, farthest first: the scope, its tokens and whether it locks.
pub(super) type Path = Vec<(i32, ScopeId, Tokens, bool)>;

/// A theme made of the defaults and the tokens set along a path, down to the first scope that locks.
pub(super) fn merged(path: &Path) -> (Theme, Option<ScopeId>) {
    let (mut merged, mut origins) = defaults();
    for (_, scope, tokens, locked) in path {
        for (name, value) in tokens
            .iter()
            .filter(|(name, _)| TOKENS.iter().any(|t| t.name == name.as_str()))
        {
            merged.insert(name.clone(), value.clone());
            origins.insert(
                name.clone(),
                Origin::Scope {
                    scope: *scope,
                    locked: *locked,
                },
            );
        }
        if *locked {
            return (Theme::new(merged, origins), Some(*scope));
        }
    }
    (Theme::new(merged, origins), None)
}

impl Customise {
    /// The theme of a scope: every token, with where it came from.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn theme(&self, scope: ScopeId) -> Result<Theme> {
        Ok(merged(&self.themes_along(scope).await?).0)
    }

    /// Sets the tokens of the theme of a scope, replacing those it set, locked or not; needs
    /// `customise.theme.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` (`customise.theme_invalid`) for an unknown
    /// token or a value of another kind, (`customise.theme_contrast`) when a text colour loses contrast on
    /// its background, `conflict` (`customise.locked_above`) below a scope that locks its theme.
    pub async fn set_theme(
        &self,
        actor: &Actor,
        scope: ScopeId,
        tokens: &Tokens,
        locked: bool,
    ) -> Result<Theme> {
        self.access.require(actor, THEME_MANAGE, scope).await?;
        let tokens = checked(tokens).map_err(|p| Error::validation("customise.theme_invalid", p))?;
        let mut path = self.themes_along(scope).await?;
        path.retain(|(_, at, _, _)| *at != scope);
        if merged(&path).1.is_some() {
            return Err(Error::conflict("customise.locked_above"));
        }
        path.push((0, scope, tokens.clone(), locked));
        let problems = low_contrast(&merged(&path).0);
        if !problems.is_empty() {
            return Err(Error::validation("customise.theme_contrast", problems));
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {t} (id, scope_id, tokens, locked) values ($1, $2, $3, $4)
             on conflict (scope_id) do update set tokens = excluded.tokens, locked = excluded.locked,
             version = {t}.version + 1",
            t = self.table("theme")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(scope.as_uuid())
        .bind(json!(tokens))
        .bind(locked)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.record_in(
            &mut tx,
            "theme",
            "theme",
            scope,
            principal(actor),
            (None, Some(json!({ "tokens": tokens, "locked": locked }))),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.theme(scope).await
    }

    /// Removes the theme a scope set, so its tokens come from above again.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `customise.theme.manage`.
    pub async fn reset_theme(&self, actor: &Actor, scope: ScopeId) -> Result<Theme> {
        self.access.require(actor, THEME_MANAGE, scope).await?;
        let mut tx = self.db.begin().await?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where scope_id = $1",
            self.table("theme")
        )))
        .bind(scope.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed > 0 {
            self.record_in(&mut tx, "theme", "theme", scope, principal(actor), (None, None))
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.theme(scope).await
    }

    /// The themes set along the path of a scope, farthest first.
    pub(super) async fn themes_along(&self, scope: ScopeId) -> Result<Path> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let depths = self.depths(&mut conn, scope).await?;
        let ids: Vec<uuid::Uuid> = depths.keys().copied().collect();
        let rows = sqlx::query(sql(format!(
            "select scope_id, tokens, locked from {} where scope_id = any($1)",
            self.table("theme")
        )))
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut path: Path = rows
            .iter()
            .map(|r| {
                let at: uuid::Uuid = r.get("scope_id");
                (
                    depths.get(&at).copied().unwrap_or(i32::MAX),
                    ScopeId::from_uuid(at),
                    serde_json::from_value(r.get("tokens")).unwrap_or_default(),
                    r.get("locked"),
                )
            })
            .collect();
        path.sort_by_key(|(depth, ..)| std::cmp::Reverse(*depth));
        Ok(path)
    }
}
