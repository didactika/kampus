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

//! The upgrade steps of the `notify` schema: the types modules declare, the notifications of each person and
//! what each person chose to receive.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// A foreign key to a table of another module, by its `id`.
const fn other(
    columns: &'static [&'static str],
    module: &'static str,
    table: &'static str,
    on_delete: OnDelete,
) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some(module),
        table,
        references: &["id"],
        on_delete,
    }
}

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[
    Step {
        sequence: 1,
        name: "notification types, notifications and preferences",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "notification_type",
                columns: &[
                    Column::new("code", Type::Text),
                    Column::new("module", Type::Text),
                    Column::new("summary", Type::Text),
                    Column::new("channels", Type::Json("notify.channels")),
                    Column::new("required", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("retired_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["code"],
                without_natural_key: Some("keyed by the code its module declares"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "notification",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("type_code", Type::Text),
                    Column::new("recipient_id", Type::Uuid),
                    Column::new("dedupe", Type::Text).nullable(),
                    Column::new("payload", Type::Json("notify.payload")),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("read_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["id"],
                unique: &[&["recipient_id", "type_code", "dedupe"]],
                foreign_keys: &[
                    ForeignKey {
                        columns: &["type_code"],
                        module: None,
                        table: "notification_type",
                        references: &["code"],
                        on_delete: OnDelete::Restrict,
                    },
                    other(&["recipient_id"], "identity", "account", OnDelete::Cascade),
                ],
                indexes: &[Index {
                    columns: &["recipient_id", "created_at"],
                    unique: false,
                }],
                without_natural_key: Some(
                    "a notification is identified by the event that caused it when there is one",
                ),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "preference",
                columns: &[
                    Column::new("account_id", Type::Uuid),
                    Column::new("type_code", Type::Text),
                    Column::new("channel", Type::Text),
                    Column::new("enabled", Type::Boolean),
                ],
                primary_key: &["account_id", "type_code", "channel"],
                foreign_keys: &[
                    other(&["account_id"], "identity", "account", OnDelete::Cascade),
                    ForeignKey {
                        columns: &["type_code"],
                        module: None,
                        table: "notification_type",
                        references: &["code"],
                        on_delete: OnDelete::Cascade,
                    },
                ],
                without_natural_key: Some("one choice per person, type and channel"),
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 2,
        name: "channels: deliveries, templates, campus defaults and push subscriptions",
        revert: &[],
        action: None,
        changes: &[
            Change::AddColumn {
                table: "notification",
                column: Column::new("in_inbox", Type::Boolean).default(ColumnDefault::Bool(true)),
            },
            Change::CreateTable(Table {
                name: "delivery",
                columns: &[
                    Column::new("notification_id", Type::Uuid),
                    Column::new("channel", Type::Text),
                    Column::new("state", Type::Text).default(ColumnDefault::Text("pending")),
                    Column::new("attempts", Type::Integer).default(ColumnDefault::Int(0)),
                    Column::new("next_attempt_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("last_error", Type::Text).nullable(),
                    Column::new("sent_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["notification_id", "channel"],
                foreign_keys: &[TYPE_NOTIFICATION],
                checks: &[
                    Check::OneOf("state", DELIVERY_STATES),
                    Check::OneOf("channel", SENT_CHANNELS),
                ],
                indexes: &[Index {
                    columns: &["state", "next_attempt_at"],
                    unique: false,
                }],
                without_natural_key: Some("one delivery per notification and channel"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "template",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("campus_id", Type::Uuid),
                    Column::new("type_code", Type::Text),
                    Column::new("channel", Type::Text),
                    Column::new("locale", Type::Text),
                    Column::new("subject", Type::Text),
                    Column::new("body", Type::Text),
                    Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["campus_id", "type_code", "channel", "locale"]],
                foreign_keys: &[
                    other(&["campus_id"], "access", "scope", OnDelete::Cascade),
                    TYPE_OF,
                ],
                checks: &[Check::OneOf("channel", SENT_CHANNELS)],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "campus_default",
                columns: &[
                    Column::new("campus_id", Type::Uuid),
                    Column::new("type_code", Type::Text),
                    Column::new("channel", Type::Text),
                    Column::new("enabled", Type::Boolean),
                    Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
                ],
                primary_key: &["campus_id", "type_code", "channel"],
                foreign_keys: &[
                    other(&["campus_id"], "access", "scope", OnDelete::Cascade),
                    TYPE_OF,
                ],
                checks: &[Check::OneOf("channel", crate::CHANNELS)],
                without_natural_key: Some("one default per campus, type and channel"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "push_subscription",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("endpoint", Type::Text),
                    Column::new("p256dh", Type::Text),
                    Column::new("auth", Type::Text),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["endpoint"]],
                foreign_keys: &[other(&["account_id"], "identity", "account", OnDelete::Cascade)],
                indexes: &[Index {
                    columns: &["account_id"],
                    unique: false,
                }],
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 3,
        name: "conversations: members, messages, deletions and blocks",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "conversation",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("campus_id", Type::Uuid),
                    Column::new("kind", Type::Text),
                    Column::new("title", Type::Text).nullable(),
                    Column::new("direct_key", Type::Text).nullable(),
                    Column::new("course_group_id", Type::Uuid).nullable(),
                    Column::new("created_by", Type::Uuid).nullable(),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["direct_key"], &["course_group_id"]],
                foreign_keys: &[
                    other(&["campus_id"], "access", "scope", OnDelete::Cascade),
                    other(&["course_group_id"], "courses", "group", OnDelete::Cascade),
                    other(&["created_by"], "identity", "account", OnDelete::SetNull),
                ],
                checks: &[Check::OneOf("kind", CONVERSATION_KINDS)],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "member",
                columns: &[
                    Column::new("conversation_id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("joined_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("left_at", Type::Timestamptz).nullable(),
                    Column::new("muted", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("last_read_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["conversation_id", "account_id"],
                foreign_keys: &[
                    CONVERSATION_OF,
                    other(&["account_id"], "identity", "account", OnDelete::Cascade),
                ],
                indexes: &[Index {
                    columns: &["account_id"],
                    unique: false,
                }],
                without_natural_key: Some("one membership per conversation and person"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "message",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("conversation_id", Type::Uuid),
                    Column::new("author_id", Type::Uuid),
                    Column::new("body", Type::Text),
                    Column::new("sent_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                foreign_keys: &[
                    CONVERSATION_OF,
                    other(&["author_id"], "identity", "account", OnDelete::Cascade),
                ],
                indexes: &[
                    Index {
                        columns: &["conversation_id", "sent_at"],
                        unique: false,
                    },
                    Index {
                        columns: &["author_id"],
                        unique: false,
                    },
                ],
                without_natural_key: Some(
                    "two messages may say the same at the same time; they differ only by id",
                ),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "message_deletion",
                columns: &[
                    Column::new("message_id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["message_id", "account_id"],
                foreign_keys: &[
                    ForeignKey {
                        columns: &["message_id"],
                        module: None,
                        table: "message",
                        references: &["id"],
                        on_delete: OnDelete::Cascade,
                    },
                    other(&["account_id"], "identity", "account", OnDelete::Cascade),
                ],
                without_natural_key: Some("one deletion per message and person"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "block",
                columns: &[
                    Column::new("account_id", Type::Uuid),
                    Column::new("blocked_id", Type::Uuid),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["account_id", "blocked_id"],
                foreign_keys: &[
                    other(&["account_id"], "identity", "account", OnDelete::Cascade),
                    other(&["blocked_id"], "identity", "account", OnDelete::Cascade),
                ],
                indexes: &[Index {
                    columns: &["blocked_id"],
                    unique: false,
                }],
                without_natural_key: Some("one block per pair of people"),
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 4,
        name: "the files attached to messages",
        revert: &[],
        action: None,
        changes: &[Change::CreateTable(Table {
            name: "message_file",
            columns: &[
                Column::new("message_id", Type::Uuid),
                Column::new("file_id", Type::Uuid),
                Column::new("position", Type::Integer),
            ],
            primary_key: &["message_id", "file_id"],
            unique: &[&["file_id"]],
            foreign_keys: &[
                ForeignKey {
                    columns: &["message_id"],
                    module: None,
                    table: "message",
                    references: &["id"],
                    on_delete: OnDelete::Cascade,
                },
                other(&["file_id"], "files", "file", OnDelete::Cascade),
            ],
            ..Table::EMPTY
        })],
    },
];

/// What a conversation is: between two people, of a group someone gathered, or of a group of a course.
pub const CONVERSATION_KINDS: &[&str] = &["direct", "group", "course_group"];

/// The foreign key to the conversation of a row.
const CONVERSATION_OF: ForeignKey = ForeignKey {
    columns: &["conversation_id"],
    module: None,
    table: "conversation",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

/// Where a delivery stands: waiting, sent, or given up after its attempts.
pub const DELIVERY_STATES: &[&str] = &["pending", "sent", "dead"];

/// The channels a delivery job sends through: every channel but the inbox.
pub const SENT_CHANNELS: &[&str] = &[crate::EMAIL, crate::PUSH];

/// The foreign key of a delivery to its notification.
const TYPE_NOTIFICATION: ForeignKey = ForeignKey {
    columns: &["notification_id"],
    module: None,
    table: "notification",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

/// The foreign key to the type of notification.
const TYPE_OF: ForeignKey = ForeignKey {
    columns: &["type_code"],
    module: None,
    table: "notification_type",
    references: &["code"],
    on_delete: OnDelete::Cascade,
};
