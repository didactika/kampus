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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `files` schema: blobs stored once by their hash with the count of the files
//! that use them, the areas modules declare, files, uploads, and the trash of blobs no file uses.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// Where the scanning of a blob stands; `unscanned` until a scanner is configured.
pub const SCAN_STATUSES: &[&str] = &["unscanned", "pending", "clean", "infected", "failed"];

/// A foreign key to a scope of access, which takes the row with it.
const SCOPE: ForeignKey = ForeignKey {
    columns: &["scope_id"],
    module: Some("access"),
    table: "scope",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

/// A foreign key to a table of this module.
const fn own(
    columns: &'static [&'static str],
    table: &'static str,
    references: &'static [&'static str],
    on_delete: OnDelete,
) -> ForeignKey {
    ForeignKey {
        columns,
        module: None,
        table,
        references,
        on_delete,
    }
}

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "blobs, areas, files, uploads and trash",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "blob",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("hash", Type::Bytes),
                Column::new("size", Type::BigInt),
                Column::new("media_type", Type::Text),
                Column::new("storage_key", Type::Text),
                Column::new("ref_count", Type::Integer).default(ColumnDefault::Int(0)),
                Column::new("scan_status", Type::Text).default(ColumnDefault::Text("unscanned")),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["hash"]],
            checks: &[
                Check::NonNegative("ref_count"),
                Check::NonNegative("size"),
                Check::OneOf("scan_status", SCAN_STATUSES),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "area",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("max_size", Type::BigInt),
                Column::new("allowed_types", Type::Json("files.allowed_types")),
                Column::new("upload_permission", Type::Text),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "file",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("blob_id", Type::Uuid),
                Column::new("area_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("name", Type::Text),
                Column::new("media_type", Type::Text),
                Column::new("size", Type::BigInt),
                Column::new("uploader", Type::Uuid).nullable(),
                Column::new("attached", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                own(&["blob_id"], "blob", &["id"], OnDelete::Restrict),
                own(&["area_code"], "area", &["code"], OnDelete::Restrict),
                SCOPE,
            ],
            indexes: &[
                Index {
                    columns: &["blob_id"],
                    unique: false,
                },
                Index {
                    columns: &["attached", "created_at"],
                    unique: false,
                },
            ],
            without_natural_key: Some("a file is known by its id; names repeat"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "upload",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("area_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("uploader", Type::Uuid),
                Column::new("name", Type::Text),
                Column::new("declared_type", Type::Text).nullable(),
                Column::new("max_size", Type::BigInt),
                Column::new("blob_id", Type::Uuid).nullable(),
                Column::new("file_id", Type::Uuid).nullable(),
                Column::new("expires_at", Type::Timestamptz),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                own(&["area_code"], "area", &["code"], OnDelete::Cascade),
                own(&["blob_id"], "blob", &["id"], OnDelete::Cascade),
                own(&["file_id"], "file", &["id"], OnDelete::Cascade),
                SCOPE,
            ],
            indexes: &[Index {
                columns: &["expires_at"],
                unique: false,
            }],
            without_natural_key: Some("an upload session is known by its id"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "trash",
            columns: &[
                Column::new("blob_id", Type::Uuid),
                Column::new("since", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["blob_id"],
            foreign_keys: &[own(&["blob_id"], "blob", &["id"], OnDelete::Cascade)],
            without_natural_key: Some("one entry per blob no file uses"),
            ..Table::EMPTY
        }),
    ],
}];
