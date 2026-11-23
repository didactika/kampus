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

//! Importing: a package is validated against the declarations, locks and permissions of the scope it goes
//! into, its differences with what the scope sets are kept for review, and applying it writes everything
//! in one transaction, recorded as one change. An import adds and changes; it never removes.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, ErrorKind, FieldError, Result, Timestamp};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};

use super::export::checksum;
use super::menu::flatten;
use super::theme::{checked, low_contrast, merged};
use super::{is_code, principal, refuse_below_lock};
use crate::model::package::{
    Difference, FORMAT, Import, ImportId, Package, PackageField, PackageKind, PackageTerm,
};
use crate::permissions::{
    FIELD_MANAGE, FLAG_MANAGE, LAYOUT_MANAGE, MENU_MANAGE, PACKAGE_MANAGE, TERM_MANAGE, THEME_MANAGE,
};
use crate::{Customise, FieldType, FieldVisibility, sql};

/// Every kind a package holds, in the order they are compared and applied.
const KINDS: &[PackageKind] = &[
    PackageKind::Terms,
    PackageKind::Flags,
    PackageKind::Fields,
    PackageKind::Layouts,
    PackageKind::Menus,
    PackageKind::Theme,
    PackageKind::Rules,
];

/// The longest form of a term, in characters.
const MAX_FORM: usize = 100;

/// Each item of a kind of a package with its key, as JSON.
fn items(package: &Package, kind: PackageKind) -> Vec<(String, Value)> {
    let keyed = |key: String, value: Value| (key, value);
    match kind {
        PackageKind::Terms => package
            .terms
            .iter()
            .map(|t| keyed(format!("{}/{}", t.code, t.language), json!(t)))
            .collect(),
        PackageKind::Flags => package
            .flags
            .iter()
            .map(|f| keyed(f.code.clone(), json!(f)))
            .collect(),
        PackageKind::Fields => package
            .fields
            .iter()
            .map(|f| keyed(format!("{}/{}", f.kind, f.spec.code), json!(f)))
            .collect(),
        PackageKind::Layouts => package
            .layouts
            .iter()
            .map(|l| keyed(l.page_kind.clone(), json!(l)))
            .collect(),
        PackageKind::Menus => package
            .menus
            .iter()
            .map(|m| keyed(m.kind.clone(), json!(m)))
            .collect(),
        PackageKind::Theme => package
            .theme
            .iter()
            .map(|t| keyed("theme".to_owned(), json!(t)))
            .collect(),
        PackageKind::Rules => package
            .rules
            .iter()
            .map(|r| keyed(r.name.trim().to_owned(), json!(r)))
            .collect(),
    }
}

/// What a package adds or changes against what a scope sets.
fn differences(package: &Package, current: &Package) -> Vec<Difference> {
    let mut found = Vec::new();
    for kind in KINDS {
        let now = items(current, *kind);
        for (key, after) in items(package, *kind) {
            let before = now.iter().find(|(k, _)| *k == key).map(|(_, v)| v.clone());
            if before.as_ref() != Some(&after) {
                found.push(Difference {
                    kind: *kind,
                    key,
                    before,
                    after,
                });
            }
        }
    }
    found
}

/// The problems of the words of terms that do not depend on the scope.
fn term_problems(at: usize, term: &PackageTerm, problems: &mut Vec<FieldError>) {
    for (name, form) in [("singular", &term.singular), ("plural", &term.plural)] {
        let form = form.trim();
        if form.is_empty() || form.chars().count() > MAX_FORM {
            problems.push(FieldError::new(format!("terms[{at}].{name}"), "length"));
        }
    }
}

/// The problems of a custom field that do not depend on the scope.
fn field_problems(at: usize, field: &PackageField, problems: &mut Vec<FieldError>) {
    let spec = &field.spec;
    if !is_code(spec.code.trim()) || spec.code.contains('.') {
        problems.push(FieldError::new(format!("fields[{at}].code"), "code"));
    }
    if spec.name.trim().is_empty() {
        problems.push(FieldError::new(format!("fields[{at}].name"), "required"));
    }
    let with_choices = matches!(spec.field_type, FieldType::Choice | FieldType::Choices);
    if with_choices == spec.choices.is_empty() {
        problems.push(FieldError::new(format!("fields[{at}].choices"), "type"));
    }
}

/// Adds a problem for something a scope above locks.
fn locked(result: &Result<()>, name: String, problems: &mut Vec<FieldError>) {
    if result.is_err() {
        problems.push(FieldError::new(name, "locked_above"));
    }
}

impl Customise {
    /// Reads a package for a scope: validates it against what the scope may set and keeps it with its
    /// differences, to be applied later; needs `customise.package.manage` and the permission of every kind
    /// it holds there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without a permission, `validation` (`customise.package_invalid`) naming every
    /// item that is not declared, does not fit, or is locked above.
    pub async fn preview_import(&self, actor: &Actor, scope: ScopeId, package: Package) -> Result<Import> {
        let problems = self.package_problems(actor, scope, &package).await?;
        if !problems.is_empty() {
            return Err(Error::validation("customise.package_invalid", problems));
        }
        let current = self.set_in(scope, KINDS).await?;
        let found = differences(&package, &current);
        let id = ImportId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, scope_id, actor, checksum, package, differences) values ($1, $2, $3, $4, $5, $6)",
            self.table("package_import")
        )))
        .bind(id.as_uuid())
        .bind(scope.as_uuid())
        .bind(principal(actor))
        .bind(checksum(&package))
        .bind(json!(package))
        .bind(json!(found))
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        self.import(id).await
    }

    /// An import with its differences.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.import_not_found`) for an unknown import.
    pub async fn import(&self, id: ImportId) -> Result<Import> {
        let row = sqlx::query(sql(format!(
            "select scope_id, checksum, differences, applied_at from {} where id = $1",
            self.table("package_import")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("customise.import_not_found"))?;
        Ok(Import {
            id,
            scope: ScopeId::from_uuid(row.get("scope_id")),
            checksum: row.get("checksum"),
            differences: serde_json::from_value(row.get("differences")).unwrap_or_default(),
            applied_at: row
                .get::<Option<time::OffsetDateTime>, _>("applied_at")
                .map(Timestamp::from_offset_date_time),
        })
    }

    /// Applies an import in one transaction, recorded as one change; it is checked again first, since the
    /// scope may have changed.
    ///
    /// # Errors
    ///
    /// Those of [`Customise::preview_import`], `not_found` for an unknown import, and `conflict`
    /// (`customise.import_applied`) for one already applied.
    pub async fn apply_import(&self, actor: &Actor, id: ImportId) -> Result<Import> {
        let stored = self.import(id).await?;
        if stored.applied_at.is_some() {
            return Err(Error::conflict("customise.import_applied"));
        }
        let package: Package = sqlx::query_scalar::<_, Value>(sql(format!(
            "select package from {} where id = $1",
            self.table("package_import")
        )))
        .bind(id.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .and_then(|v| {
            serde_json::from_value(v).map_err(|_| Error::internal("customise.import_unreadable"))
        })?;
        let problems = self.package_problems(actor, stored.scope, &package).await?;
        if !problems.is_empty() {
            return Err(Error::validation("customise.package_invalid", problems));
        }
        let owner = principal(actor).ok_or_else(|| Error::permission_denied("customise.rule_owner"))?;
        let mut tx = self.db.begin().await?;
        let applied = sqlx::query(sql(format!(
            "update {} set applied_at = now() where id = $1 and applied_at is null",
            self.table("package_import")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if applied == 0 {
            return Err(Error::conflict("customise.import_applied"));
        }
        self.write_package_in(&mut tx, stored.scope, &package, owner)
            .await?;
        let summary = json!({
            "terms": package.terms.len(), "flags": package.flags.len(), "fields": package.fields.len(),
            "layouts": package.layouts.len(), "menus": package.menus.len(),
            "theme": package.theme.is_some(), "rules": package.rules.len(),
        });
        self.record_in(
            &mut tx,
            "package",
            &stored.checksum,
            stored.scope,
            Some(owner),
            (None, Some(summary)),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.import(id).await
    }

    /// The problems of a package in a scope; fails without a permission it needs.
    async fn package_problems(
        &self,
        actor: &Actor,
        scope: ScopeId,
        package: &Package,
    ) -> Result<Vec<FieldError>> {
        self.access.require(actor, PACKAGE_MANAGE, scope).await?;
        let needed = [
            (!package.terms.is_empty(), TERM_MANAGE),
            (!package.flags.is_empty(), FLAG_MANAGE),
            (!package.fields.is_empty(), FIELD_MANAGE),
            (!package.layouts.is_empty(), LAYOUT_MANAGE),
            (!package.menus.is_empty(), MENU_MANAGE),
            (package.theme.is_some(), THEME_MANAGE),
        ];
        for (_, permission) in needed.iter().filter(|(used, _)| *used) {
            self.access.require(actor, permission, scope).await?;
        }
        let mut problems = Vec::new();
        if package.format != FORMAT {
            problems.push(FieldError::new("format", "one_of"));
        }
        self.declared_problems(scope, package, &mut problems).await?;
        self.page_problems(scope, package, &mut problems).await?;
        for (at, rule) in package.rules.iter().enumerate() {
            match self.checked_rule(actor, scope, rule).await {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::Validation => problems.extend(
                    e.fields()
                        .iter()
                        .map(|f| FieldError::new(format!("rules[{at}].{}", f.field), f.rule.clone())),
                ),
                Err(e) => return Err(e),
            }
        }
        Ok(problems)
    }

    /// The problems of the terms, flags and fields of a package in a scope.
    async fn declared_problems(
        &self,
        scope: ScopeId,
        package: &Package,
        problems: &mut Vec<FieldError>,
    ) -> Result<()> {
        for (at, term) in package.terms.iter().enumerate() {
            term_problems(at, term, problems);
            match self.term(&term.code, scope, &term.language).await {
                Ok(current) => locked(
                    &refuse_below_lock(current.origin, scope),
                    format!("terms[{at}]"),
                    problems,
                ),
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    problems.push(FieldError::new(format!("terms[{at}].code"), "one_of"));
                }
                Err(e) => return Err(e),
            }
        }
        for (at, flag) in package.flags.iter().enumerate() {
            match self.flag(&flag.code, scope).await {
                Ok(current) => locked(
                    &refuse_below_lock(current.origin, scope),
                    format!("flags[{at}]"),
                    problems,
                ),
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    problems.push(FieldError::new(format!("flags[{at}].code"), "one_of"));
                }
                Err(e) => return Err(e),
            }
        }
        for (at, field) in package.fields.iter().enumerate() {
            field_problems(at, field, problems);
            let row = sqlx::query(sql(format!(
                "select k.code, f.type from {} k left join {} f on f.kind_code = k.code and f.scope_id = $2
                 and f.code = $3 where k.code = $1 and k.retired_at is null",
                self.table("field_kind"),
                self.table("field")
            )))
            .bind(&field.kind)
            .bind(scope.as_uuid())
            .bind(field.spec.code.trim())
            .fetch_optional(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            match row.map(|r| r.get::<Option<String>, _>("type")) {
                None => problems.push(FieldError::new(format!("fields[{at}].kind"), "one_of")),
                Some(Some(stored)) if stored != field.spec.field_type.as_str() => {
                    problems.push(FieldError::new(format!("fields[{at}].type"), "unchangeable"));
                }
                Some(_) => {}
            }
        }
        Ok(())
    }

    /// The problems of the layouts, menus and theme of a package in a scope.
    async fn page_problems(
        &self,
        scope: ScopeId,
        package: &Package,
        problems: &mut Vec<FieldError>,
    ) -> Result<()> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        for (at, layout) in package.layouts.iter().enumerate() {
            let grid = match self.grid(&mut conn, &layout.page_kind).await {
                Ok(grid) => grid,
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    problems.push(FieldError::new(format!("layouts[{at}].page_kind"), "one_of"));
                    continue;
                }
                Err(e) => return Err(e),
            };
            let widgets = self.widgets(&mut conn, &layout.placements).await?;
            problems.extend(
                super::layout::problems(&layout.page_kind, grid, &widgets, &layout.placements)
                    .into_iter()
                    .map(|p| FieldError::new(format!("layouts[{at}].{}", p.field), p.rule)),
            );
            if let Some((_, origin)) = self
                .nearest(&mut conn, ("layout", "page_kind_code"), &layout.page_kind, scope)
                .await?
            {
                locked(
                    &refuse_below_lock(origin, scope),
                    format!("layouts[{at}]"),
                    problems,
                );
            }
        }
        for (at, menu) in package.menus.iter().enumerate() {
            if self.require_menu_kind(&mut conn, &menu.kind).await.is_err() {
                problems.push(FieldError::new(format!("menus[{at}].kind"), "one_of"));
                continue;
            }
            problems.extend(
                flatten(&menu.items)
                    .1
                    .into_iter()
                    .map(|p| FieldError::new(format!("menus[{at}].{}", p.field), p.rule)),
            );
            if let Some((_, origin)) = self
                .nearest(&mut conn, ("menu", "menu_kind_code"), &menu.kind, scope)
                .await?
            {
                locked(
                    &refuse_below_lock(origin, scope),
                    format!("menus[{at}]"),
                    problems,
                );
            }
        }
        drop(conn);
        if let Some(theme) = &package.theme {
            match checked(&theme.tokens) {
                Err(found) => problems.extend(
                    found
                        .into_iter()
                        .map(|p| FieldError::new(format!("theme.{}", p.field), p.rule)),
                ),
                Ok(tokens) => {
                    let mut path = self.themes_along(scope).await?;
                    path.retain(|(_, at, _, _)| *at != scope);
                    if merged(&path).1.is_some() {
                        problems.push(FieldError::new("theme", "locked_above"));
                    }
                    path.push((0, scope, tokens, theme.locked));
                    problems.extend(
                        low_contrast(&merged(&path).0)
                            .into_iter()
                            .map(|p| FieldError::new(format!("theme.{}", p.field), p.rule)),
                    );
                }
            }
        }
        Ok(())
    }

    /// Writes every item of a package in a scope, inside the transaction of the import.
    async fn write_package_in(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        package: &Package,
        owner: uuid::Uuid,
    ) -> Result<()> {
        for term in &package.terms {
            sqlx::query(sql(format!(
                "insert into {t} (term_code, scope_id, language_code, singular, plural, locked) values ($1, $2, $3, $4, $5, $6)
                 on conflict (term_code, scope_id, language_code) do update set singular = excluded.singular,
                 plural = excluded.plural, locked = excluded.locked, version = {t}.version + 1",
                t = self.table("term_override")
            )))
            .bind(&term.code)
            .bind(scope.as_uuid())
            .bind(&term.language)
            .bind(term.singular.trim())
            .bind(term.plural.trim())
            .bind(term.locked)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        for flag in &package.flags {
            sqlx::query(sql(format!(
                "insert into {} (flag_code, scope_id, enabled, locked) values ($1, $2, $3, $4)
                 on conflict (flag_code, scope_id) do update set enabled = excluded.enabled, locked = excluded.locked",
                self.table("flag_value")
            )))
            .bind(&flag.code)
            .bind(scope.as_uuid())
            .bind(flag.enabled)
            .bind(flag.locked)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        for field in &package.fields {
            self.write_field_in(conn, scope, field).await?;
        }
        for layout in &package.layouts {
            self.write_layout_in(conn, &layout.page_kind, scope, &layout.placements, layout.locked)
                .await?;
        }
        for menu in &package.menus {
            let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
                "insert into {t} (id, menu_kind_code, scope_id, locked) values ($1, $2, $3, $4)
                 on conflict (menu_kind_code, scope_id) do update set locked = excluded.locked,
                 version = {t}.version + 1 returning id",
                t = self.table("menu")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(&menu.kind)
            .bind(scope.as_uuid())
            .bind(menu.locked)
            .fetch_one(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
            self.write_items_in(conn, id, &flatten(&menu.items).0).await?;
        }
        if let Some(theme) = &package.theme {
            let tokens =
                checked(&theme.tokens).map_err(|p| Error::validation("customise.package_invalid", p))?;
            sqlx::query(sql(format!(
                "insert into {t} (id, scope_id, tokens, locked) values ($1, $2, $3, $4)
                 on conflict (scope_id) do update set tokens = excluded.tokens, locked = excluded.locked,
                 version = {t}.version + 1",
                t = self.table("theme")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(scope.as_uuid())
            .bind(json!(tokens))
            .bind(theme.locked)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        for rule in &package.rules {
            self.write_rule_in(conn, scope, rule, owner).await?;
        }
        Ok(())
    }

    /// Adds a custom field to a scope, or changes the one it has with that code, with its choices.
    async fn write_field_in(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        field: &PackageField,
    ) -> Result<()> {
        let spec = &field.spec;
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {t} (id, kind_code, scope_id, code, name, type, required, visibility, max_length, sort_key)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $1::text)
             on conflict (kind_code, scope_id, code) do update set name = excluded.name, required = excluded.required,
             visibility = excluded.visibility, max_length = excluded.max_length, version = {t}.version + 1
             returning id",
            t = self.table("field")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(&field.kind)
        .bind(scope.as_uuid())
        .bind(spec.code.trim())
        .bind(spec.name.trim())
        .bind(spec.field_type.as_str())
        .bind(spec.required)
        .bind(match spec.visibility {
            FieldVisibility::Public => "public",
            FieldVisibility::Private => "private",
        })
        .bind(spec.max_length)
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for (position, choice) in spec.choices.iter().enumerate() {
            sqlx::query(sql(format!(
                "insert into {} (id, field_id, code, label, sort_key) values ($1, $2, $3, $4, $5)
                 on conflict (field_id, code) do update set label = excluded.label, sort_key = excluded.sort_key",
                self.table("field_choice")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(id)
            .bind(choice.code.trim())
            .bind(choice.label.trim())
            .bind(format!("{position:06}"))
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    }

    /// Changes the rule of a scope with a name, or adds it; the importer becomes its owner.
    async fn write_rule_in(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        rule: &crate::RuleSpec,
        owner: uuid::Uuid,
    ) -> Result<()> {
        let conditions = rule.conditions.as_ref().map(|c| json!(c));
        let changed = sqlx::query(sql(format!(
            "update {} set owner = $3, trigger = $4, conditions = $5, actions = $6, enabled = $7, version = version + 1
             where scope_id = $1 and name = $2",
            self.table("rule")
        )))
        .bind(scope.as_uuid())
        .bind(rule.name.trim())
        .bind(owner)
        .bind(&rule.trigger)
        .bind(&conditions)
        .bind(json!(rule.actions))
        .bind(rule.enabled)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            sqlx::query(sql(format!(
                "insert into {} (id, scope_id, owner, name, trigger, conditions, actions, enabled)
                 values ($1, $2, $3, $4, $5, $6, $7, $8)",
                self.table("rule")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(scope.as_uuid())
            .bind(owner)
            .bind(rule.name.trim())
            .bind(&rule.trigger)
            .bind(&conditions)
            .bind(json!(rule.actions))
            .bind(rule.enabled)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    }
}
