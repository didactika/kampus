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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The directory of a campus: its accounts a page at a time, newest first, found by words of their username
//! or email and kept by status and kind, for whoever reads the accounts of the campus.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Cursor, Error, FieldError, Page, PageRequest, Result};

use crate::permissions::ACCOUNT_READ;
use crate::service::account::{COLUMNS, from_row};
use crate::{Account, AccountKind, AccountStatus, Identity, sql};

/// What a list of accounts keeps.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountFilter {
    /// Words found in the username or the email.
    pub words: Option<String>,
    /// Only the accounts in this status.
    pub status: Option<AccountStatus>,
    /// Only the accounts of this kind.
    pub kind: Option<AccountKind>,
}

/// A pattern of `ilike` that finds words as they are written, whatever they hold.
fn containing(words: &str) -> String {
    let escaped = words
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

impl Identity {
    /// A page of the accounts of a campus, newest first, that a filter keeps.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `identity.account.read` in the campus, `validation`
    /// (`common.page_invalid`) for a cursor of another list.
    pub async fn accounts(
        &self,
        actor: &Actor,
        campus: ScopeId,
        filter: &AccountFilter,
        page: &PageRequest,
    ) -> Result<Page<Account>> {
        self.access.require(actor, ACCOUNT_READ, campus).await?;
        let after = page
            .after()
            .map(|c| uuid::Uuid::from_slice(c.as_bytes()))
            .transpose()
            .map_err(|_| {
                Error::validation("common.page_invalid", vec![FieldError::new("after", "cursor")])
            })?;
        let limit = usize::try_from(page.limit()).unwrap_or(usize::MAX);
        let words = filter
            .words
            .as_deref()
            .filter(|w| !w.trim().is_empty())
            .map(containing);
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where campus_id = $1
               and ($2::uuid is null or id < $2)
               and ($3::text is null or username ilike $3 or email ilike $3)
               and ($4::text is null or status = $4)
               and ($5::text is null or kind = $5)
             order by id desc limit $6",
            self.table("account")
        )))
        .bind(campus.as_uuid())
        .bind(after)
        .bind(words)
        .bind(filter.status.map(AccountStatus::as_str))
        .bind(filter.kind.map(AccountKind::as_str))
        .bind(i64::from(page.limit()) + 1)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut items = rows.iter().map(from_row).collect::<Result<Vec<_>>>()?;
        let next = (items.len() > limit).then(|| {
            items.truncate(limit);
            Cursor::from_bytes(
                items
                    .last()
                    .map_or_else(Vec::new, |a| a.id.as_uuid().as_bytes().to_vec()),
            )
        });
        Ok(Page { items, next })
    }
}
