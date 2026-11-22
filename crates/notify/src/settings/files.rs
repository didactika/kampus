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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The place where notify keeps files: those attached to messages, which the members of their conversation
//! read.

use kampus_module::{FileAreaDeclaration, FileRead, ReadFuture};

use crate::permissions::MESSAGE_ATTACH;
use crate::{Notify, sql};

/// The area of the files of messages.
pub const MESSAGE_AREA: &str = "notify.message";

/// The file areas of this module.
pub const FILE_AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: MESSAGE_AREA,
    summary: "The files attached to a message",
    max_size: 10 * 1024 * 1024,
    allowed_types: &[
        "image/*",
        "application/pdf",
        "text/plain",
        "text/csv",
        "application/zip",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "application/vnd.oasis.opendocument.text",
        "application/vnd.oasis.opendocument.spreadsheet",
        "application/vnd.oasis.opendocument.presentation",
    ],
    upload_permission: MESSAGE_ATTACH,
    readable: message_file_readable,
}];

/// A file of a message is read by whoever takes part in its conversation now.
fn message_file_readable(read: FileRead<'_>) -> ReadFuture<'_> {
    Box::pin(async move {
        let Some(principal) = read.principal else {
            return Ok(false);
        };
        let notify = read.services.service::<Notify>()?;
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} mf join {} m on m.id = mf.message_id
                 join {} mb on mb.conversation_id = m.conversation_id
                 where mf.file_id = $1 and mb.account_id = $2 and mb.left_at is null)",
            notify.table("message_file"),
            notify.table("message"),
            notify.table("member")
        )))
        .bind(read.file)
        .bind(principal)
        .fetch_one(notify.db.pool())
        .await
        .map_err(kampus_db::classify)
    })
}
