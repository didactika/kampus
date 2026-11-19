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

//! Exporting: what a scope itself sets of the kinds chosen, never what it takes from above, as a package
//! with the checksum of its contents.

use std::collections::BTreeMap;
use std::fmt::Write;

use kampus_access::{Actor, ScopeId};
use kampus_common::Result;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::Row;

use super::menu::{Stored, tree};
use crate::model::package::{
    Exported, Package, PackageField, PackageFlag, PackageKind, PackageLayout, PackageMenu, PackageTerm,
    PackageTheme,
};
use crate::permissions::PACKAGE_MANAGE;
use crate::{Customise, FieldSpec, MenuItem, Placement, RuleSpec, sql};

/// The SHA-256 of a package as written, in hexadecimal.
pub(crate) fn checksum(package: &Package) -> String {
    let bytes = serde_json::to_vec(package).unwrap_or_default();
    Sha256::digest(&bytes).iter().fold(String::new(), |mut hex, b| {
        let _ = write!(hex, "{b:02x}");
        hex
    })
}

impl Customise {
    /// Exports what a scope sets of the kinds chosen; needs `customise.package.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn export(&self, actor: &Actor, scope: ScopeId, kinds: &[PackageKind]) -> Result<Exported> {
        self.access.require(actor, PACKAGE_MANAGE, scope).await?;
        let package = self.set_in(scope, kinds).await?;
        Ok(Exported {
            checksum: checksum(&package),
            package,
        })
    }

    /// What a scope sets of the kinds chosen, as a package.
    pub(super) async fn set_in(&self, scope: ScopeId, kinds: &[PackageKind]) -> Result<Package> {
        let mut package = Package::empty();
        for kind in kinds {
            match kind {
                PackageKind::Terms => package.terms = self.terms_set_in(scope).await?,
                PackageKind::Flags => package.flags = self.flags_set_in(scope).await?,
                PackageKind::Fields => package.fields = self.fields_set_in(scope).await?,
                PackageKind::Layouts => package.layouts = self.layouts_set_in(scope).await?,
                PackageKind::Menus => package.menus = self.menus_set_in(scope).await?,
                PackageKind::Theme => package.theme = self.theme_set_in(scope).await?,
                PackageKind::Rules => package.rules = self.rules_set_in(scope).await?,
            }
        }
        Ok(package)
    }

    /// The rows a scope sets in a table, in an order.
    async fn rows_in(
        &self,
        table: &str,
        columns: &str,
        order: &str,
        scope: ScopeId,
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        sqlx::query(sql(format!(
            "select {columns} from {} where scope_id = $1 order by {order}",
            self.table(table)
        )))
        .bind(scope.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    /// The words a scope gives terms.
    async fn terms_set_in(&self, scope: ScopeId) -> Result<Vec<PackageTerm>> {
        let rows = self
            .rows_in(
                "term_override",
                "term_code, language_code, singular, plural, locked",
                "term_code, language_code",
                scope,
            )
            .await?;
        Ok(rows
            .iter()
            .map(|r| PackageTerm {
                code: r.get("term_code"),
                language: r.get("language_code"),
                singular: r.get("singular"),
                plural: r.get("plural"),
                locked: r.get("locked"),
            })
            .collect())
    }

    /// The values a scope gives feature flags.
    async fn flags_set_in(&self, scope: ScopeId) -> Result<Vec<PackageFlag>> {
        let rows = self
            .rows_in("flag_value", "flag_code, enabled, locked", "flag_code", scope)
            .await?;
        Ok(rows
            .iter()
            .map(|r| PackageFlag {
                code: r.get("flag_code"),
                enabled: r.get("enabled"),
                locked: r.get("locked"),
            })
            .collect())
    }

    /// The custom fields a scope defines.
    async fn fields_set_in(&self, scope: ScopeId) -> Result<Vec<PackageField>> {
        let rows = sqlx::query(sql(format!(
            "select {} from {} f where f.scope_id = $1 order by f.kind_code, f.sort_key, f.id",
            super::field::COLUMNS,
            self.table("field")
        )))
        .bind(scope.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(self
            .with_choices(rows)
            .await?
            .into_iter()
            .map(|f| PackageField {
                kind: f.kind,
                spec: FieldSpec {
                    code: f.code,
                    name: f.name,
                    field_type: f.field_type,
                    required: f.required,
                    visibility: f.visibility,
                    max_length: f.max_length,
                    choices: f.choices,
                },
            })
            .collect())
    }

    /// The layouts a scope gives page kinds, with every placement whoever sees it.
    async fn layouts_set_in(&self, scope: ScopeId) -> Result<Vec<PackageLayout>> {
        let layouts = self
            .rows_in("layout", "id, page_kind_code, locked", "page_kind_code", scope)
            .await?;
        let mut exported = Vec::new();
        for layout in &layouts {
            let rows = sqlx::query(sql(format!(
                "select region, widget_code, settings, permission from {} where layout_id = $1 order by position",
                self.table("placement")
            )))
            .bind(layout.get::<uuid::Uuid, _>("id"))
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            exported.push(PackageLayout {
                page_kind: layout.get("page_kind_code"),
                placements: rows
                    .iter()
                    .map(|r| Placement {
                        region: r.get("region"),
                        widget: r.get("widget_code"),
                        settings: serde_json::from_value::<BTreeMap<String, Value>>(r.get("settings"))
                            .unwrap_or_default(),
                        permission: r.get("permission"),
                    })
                    .collect(),
                locked: layout.get("locked"),
            });
        }
        Ok(exported)
    }

    /// The menus a scope gives kinds of menu, with every item whoever sees it.
    async fn menus_set_in(&self, scope: ScopeId) -> Result<Vec<PackageMenu>> {
        let menus = self
            .rows_in("menu", "id, menu_kind_code, locked", "menu_kind_code", scope)
            .await?;
        let mut exported = Vec::new();
        for menu in &menus {
            let rows = sqlx::query(sql(format!(
                "select id, parent_id, label, target, permission from {} where menu_id = $1 order by position",
                self.table("menu_item")
            )))
            .bind(menu.get::<uuid::Uuid, _>("id"))
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            let stored: Vec<Stored> = rows
                .iter()
                .map(|r| Stored {
                    id: r.get("id"),
                    parent: r.get("parent_id"),
                    item: MenuItem {
                        label: r.get("label"),
                        target: r.get("target"),
                        permission: r.get("permission"),
                        children: Vec::new(),
                    },
                })
                .collect();
            exported.push(PackageMenu {
                kind: menu.get("menu_kind_code"),
                items: tree(&stored, None),
                locked: menu.get("locked"),
            });
        }
        Ok(exported)
    }

    /// The tokens a scope sets in its theme.
    async fn theme_set_in(&self, scope: ScopeId) -> Result<Option<PackageTheme>> {
        let rows = self.rows_in("theme", "tokens, locked", "id", scope).await?;
        Ok(rows.first().map(|r| PackageTheme {
            tokens: serde_json::from_value(r.get("tokens")).unwrap_or_default(),
            locked: r.get("locked"),
        }))
    }

    /// The rules a scope writes.
    async fn rules_set_in(&self, scope: ScopeId) -> Result<Vec<RuleSpec>> {
        Ok(self
            .rules(scope)
            .await?
            .into_iter()
            .map(|r| RuleSpec {
                name: r.name,
                trigger: r.trigger,
                conditions: r.conditions,
                actions: r.actions,
                enabled: r.enabled,
            })
            .collect())
    }
}
