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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `privacy` schema: the requests of people for their personal data, the part of
//! each request that every module answers, and the policies of a campus with who accepted which version.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// What a request asks for.
pub const KINDS: &[&str] = &["export", "deletion"];

/// Where a request stands.
pub const STATES: &[&str] = &["pending", "completed", "failed", "expired"];

/// Where one part of a request stands.
pub const ITEM_STATES: &[&str] = &["pending", "completed", "failed"];

/// A foreign key to the campus, which takes the row with it.
const CAMPUS: ForeignKey = ForeignKey {
    columns: &["campus_id"],
    module: Some("access"),
    table: "scope",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[
    Step {
        sequence: 1,
        name: "requests and their parts",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "request",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("requester", Type::Uuid).nullable(),
                    Column::new("kind", Type::Text),
                    Column::new("state", Type::Text).default(ColumnDefault::Text("pending")),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("due_at", Type::Timestamptz),
                    Column::new("completed_at", Type::Timestamptz).nullable(),
                    Column::new("expires_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["id"],
                foreign_keys: &[ForeignKey {
                    columns: &["account_id"],
                    module: Some("identity"),
                    table: "account",
                    references: &["id"],
                    on_delete: OnDelete::Cascade,
                }],
                checks: &[Check::OneOf("kind", KINDS), Check::OneOf("state", STATES)],
                indexes: &[Index {
                    columns: &["account_id", "created_at"],
                    unique: false,
                }],
                without_natural_key: Some("a person may ask as often as they need"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "request_item",
                columns: &[
                    Column::new("request_id", Type::Uuid),
                    Column::new("category", Type::Text),
                    Column::new("state", Type::Text).default(ColumnDefault::Text("pending")),
                    Column::new("data", Type::Json("privacy.export")).nullable(),
                    Column::new("error", Type::Text).nullable(),
                ],
                primary_key: &["request_id", "category"],
                foreign_keys: &[ForeignKey {
                    columns: &["request_id"],
                    module: None,
                    table: "request",
                    references: &["id"],
                    on_delete: OnDelete::Cascade,
                }],
                checks: &[Check::OneOf("state", ITEM_STATES)],
                without_natural_key: Some("one part per data category of a request"),
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 2,
        name: "policies and acceptances",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "policy",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("campus_id", Type::Uuid),
                    Column::new("code", Type::Text),
                    Column::new("title", Type::Text),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["campus_id", "code"]],
                foreign_keys: &[CAMPUS],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "policy_version",
                columns: &[
                    Column::new("policy_id", Type::Uuid),
                    Column::new("version", Type::Integer),
                    Column::new("body", Type::Text),
                    Column::new("published_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("published_by", Type::Uuid).nullable(),
                ],
                primary_key: &["policy_id", "version"],
                foreign_keys: &[ForeignKey {
                    columns: &["policy_id"],
                    module: None,
                    table: "policy",
                    references: &["id"],
                    on_delete: OnDelete::Cascade,
                }],
                without_natural_key: Some("one row per published version of a policy"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "acceptance",
                columns: &[
                    Column::new("policy_id", Type::Uuid),
                    Column::new("version", Type::Integer),
                    Column::new("account_id", Type::Uuid),
                    Column::new("accepted_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["policy_id", "version", "account_id"],
                foreign_keys: &[
                    ForeignKey {
                        columns: &["policy_id", "version"],
                        module: None,
                        table: "policy_version",
                        references: &["policy_id", "version"],
                        on_delete: OnDelete::Cascade,
                    },
                    ForeignKey {
                        columns: &["account_id"],
                        module: Some("identity"),
                        table: "account",
                        references: &["id"],
                        on_delete: OnDelete::Cascade,
                    },
                ],
                without_natural_key: Some("one acceptance per person and version, the target of the upsert"),
                ..Table::EMPTY
            }),
        ],
    },
];
