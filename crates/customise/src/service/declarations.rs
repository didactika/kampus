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

//! What modules declare for customisation: their terms, their feature flags and the kinds of entity that
//! accept custom fields; what no module declares any more is retired, and the customisations stay.

use kampus_common::Result;
use kampus_module::Module;
use serde_json::json;

use crate::{Customise, sql};

impl Customise {
    /// Brings the terms, flags and field kinds the modules of the build declare into the database.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sync_declarations(&self, modules: &[&Module]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let (mut terms, mut flags, mut kinds) = (Vec::new(), Vec::new(), Vec::new());
        for module in modules {
            for term in module.terms {
                terms.push(term.code.to_owned());
                let forms: Vec<_> = term
                    .forms
                    .iter()
                    .map(|f| json!({ "language": f.language, "singular": f.singular, "plural": f.plural }))
                    .collect();
                sqlx::query(sql(format!(
                    "insert into {} (code, module, forms) values ($1, $2, $3)
                     on conflict (code) do update set module = excluded.module, forms = excluded.forms, retired_at = null",
                    self.table("term")
                )))
                .bind(term.code)
                .bind(module.name)
                .bind(json!(forms))
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
            for flag in module.flags {
                flags.push(flag.code.to_owned());
                sqlx::query(sql(format!(
                    "insert into {} (code, module, summary, default_value) values ($1, $2, $3, $4)
                     on conflict (code) do update set module = excluded.module, summary = excluded.summary,
                     default_value = excluded.default_value, retired_at = null",
                    self.table("flag")
                )))
                .bind(flag.code)
                .bind(module.name)
                .bind(flag.summary)
                .bind(flag.default)
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
            for kind in module.field_kinds {
                kinds.push(kind.code.to_owned());
                sqlx::query(sql(format!(
                    "insert into {} (code, module, summary) values ($1, $2, $3)
                     on conflict (code) do update set module = excluded.module, summary = excluded.summary, retired_at = null",
                    self.table("field_kind")
                )))
                .bind(kind.code)
                .bind(module.name)
                .bind(kind.summary)
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        for (table, codes) in [("term", &terms), ("flag", &flags), ("field_kind", &kinds)] {
            sqlx::query(sql(format!(
                "update {} set retired_at = now() where not (code = any($1)) and retired_at is null",
                self.table(table)
            )))
            .bind(codes)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }
}
