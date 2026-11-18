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

//! The upgrade steps of the `identity` schema, in the neutral schema. Usernames and emails are
//! stored lower-case, so plain unique keys give case-insensitive uniqueness.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The statuses of an account, as stored.
pub const STATUSES: &[&str] = &[
    "invited",
    "pending_confirmation",
    "active",
    "suspended",
    "deleted",
];

/// The kinds of account, as stored.
const ACCOUNT_KINDS: &[&str] = &["person", "service"];

/// The purposes of a one-time token, as stored.
const PURPOSES: &[&str] = &[
    "confirm_email",
    "invitation",
    "set_password",
    "recovery",
    "email_code",
    "second_factor",
];

/// The outcomes of a sign-in attempt, as stored.
const OUTCOMES: &[&str] = &["success", "failure", "throttled", "second_factor_required"];

/// A foreign key to a table of this module.
const fn local(columns: &'static [&'static str], table: &'static str, on_delete: OnDelete) -> ForeignKey {
    ForeignKey {
        columns,
        module: None,
        table,
        references: &["id"],
        on_delete,
    }
}

/// A foreign key to a table of access.
const fn access(columns: &'static [&'static str], table: &'static str, on_delete: OnDelete) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some("access"),
        table,
        references: &["id"],
        on_delete,
    }
}

/// The owner of a row: its account, which takes the row with it.
const OWNER: ForeignKey = local(&["account_id"], "account", OnDelete::Cascade);

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "accounts, credentials, sessions and tokens",
    changes: &[
        Change::CreateTable(Table {
            name: "account",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("campus_id", Type::Uuid),
                Column::new("scope_id", Type::Uuid),
                Column::new("kind", Type::Text),
                Column::new("username", Type::Text).nullable(),
                Column::new("email", Type::Text).nullable(),
                Column::new("email_verified_at", Type::Timestamptz).nullable(),
                Column::new("status", Type::Text),
                Column::new("status_reason", Type::Text).nullable(),
                Column::new("status_changed_at", Type::Timestamptz),
                Column::new("status_changed_by", Type::Uuid).nullable(),
                Column::new("last_sign_in_at", Type::Timestamptz).nullable(),
                Column::new("pseudonym", Type::Text).nullable(),
                Column::new("deletion_due_at", Type::Timestamptz).nullable(),
                Column::new("deleted_at", Type::Timestamptz).nullable(),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["scope_id"], &["campus_id", "username"], &["campus_id", "email"]],
            foreign_keys: &[
                access(&["id"], "principal", OnDelete::Cascade),
                access(&["campus_id"], "scope", OnDelete::Restrict),
                access(&["scope_id"], "scope", OnDelete::Restrict),
            ],
            checks: &[
                Check::OneOf("status", STATUSES),
                Check::OneOf("kind", ACCOUNT_KINDS),
            ],
            indexes: &[Index {
                columns: &["deletion_due_at"],
                unique: false,
            }],
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "alter table {schema}.\"account\" add constraint \"account_tombstone_check\" check ((\"status\" = 'deleted') = (\"username\" is null) and (\"status\" = 'deleted') = (\"pseudonym\" is not null));\nalter table {schema}.\"account\" add constraint \"account_email_check\" check (\"kind\" = 'service' or \"status\" = 'deleted' or \"email\" is not null);",
            reason: "the neutral schema has no condition between two columns",
        },
        Change::CreateTable(Table {
            name: "account_transition",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("from_status", Type::Text).nullable(),
                Column::new("to_status", Type::Text),
                Column::new("reason", Type::Text).nullable(),
                Column::new("actor", Type::Uuid).nullable(),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[OWNER],
            checks: &[Check::OneOf("to_status", STATUSES)],
            without_natural_key: Some("an append-only history of the account lifecycle"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "password",
            columns: &[
                Column::new("account_id", Type::Uuid),
                Column::new("hash", Type::Text),
                Column::new("algorithm_params", Type::Text),
                Column::new("set_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("must_change", Type::Boolean).default(ColumnDefault::Bool(false)),
            ],
            primary_key: &["account_id"],
            foreign_keys: &[OWNER],
            without_natural_key: Some("one row per account, keyed by the account"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "passkey",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("credential", Type::Bytes),
                Column::new("public_key", Type::Bytes),
                Column::new("sign_count", Type::BigInt).default(ColumnDefault::Int(0)),
                Column::new("label", Type::Text),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("last_used_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["credential"]],
            foreign_keys: &[OWNER],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "identity_provider",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("campus_id", Type::Uuid),
                Column::new("kind", Type::Text),
                Column::new("issuer", Type::Text),
                Column::new("client", Type::Text),
                Column::new("client_secret_encrypted", Type::Bytes),
                Column::new("claim_mapping", Type::Json("identity.claim_mapping")),
                Column::new("linking_policy", Type::Text),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["campus_id", "issuer"]],
            foreign_keys: &[access(&["campus_id"], "scope", OnDelete::Cascade)],
            checks: &[
                Check::OneOf("kind", &["oidc"]),
                Check::OneOf("linking_policy", &["subject_only", "verified_email", "confirm"]),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "external_identity",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("provider_id", Type::Uuid),
                Column::new("subject", Type::Text),
                Column::new("email_at_link", Type::Text).nullable(),
                Column::new("linked_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("last_used_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["provider_id", "subject"]],
            foreign_keys: &[
                OWNER,
                local(&["provider_id"], "identity_provider", OnDelete::Cascade),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "second_factor",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("kind", Type::Text),
                Column::new("secret_encrypted", Type::Bytes).nullable(),
                Column::new("passkey_id", Type::Uuid).nullable(),
                Column::new("label", Type::Text),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("last_used_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["account_id", "label"]],
            foreign_keys: &[OWNER, local(&["passkey_id"], "passkey", OnDelete::Cascade)],
            checks: &[Check::OneOf("kind", &["totp", "passkey", "email"])],
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "alter table {schema}.\"second_factor\" add constraint \"second_factor_kind_columns_check\" check ((\"kind\" = 'totp') = (\"secret_encrypted\" is not null) and (\"kind\" = 'passkey') = (\"passkey_id\" is not null));",
            reason: "the neutral schema has no condition between two columns",
        },
        Change::CreateTable(Table {
            name: "recovery_code",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("hash", Type::Bytes),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("used_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["account_id", "hash"]],
            foreign_keys: &[OWNER],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "one_time_token",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("purpose", Type::Text),
                Column::new("hash", Type::Bytes),
                Column::new("created_at", Type::Timestamptz),
                Column::new("expires_at", Type::Timestamptz),
                Column::new("used_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["hash"]],
            foreign_keys: &[OWNER],
            checks: &[
                Check::OneOf("purpose", PURPOSES),
                Check::After("expires_at", "created_at"),
            ],
            indexes: &[Index {
                columns: &["expires_at"],
                unique: false,
            }],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "session",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("token_hash", Type::Bytes),
                Column::new("mfa_level", Type::Text),
                Column::new("strong_auth_at", Type::Timestamptz),
                Column::new("created_at", Type::Timestamptz),
                Column::new("last_seen_at", Type::Timestamptz),
                Column::new("expires_at", Type::Timestamptz),
                Column::new("client", Type::Text).nullable(),
                Column::new("address", Type::Text).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["token_hash"]],
            foreign_keys: &[OWNER],
            checks: &[
                Check::OneOf("mfa_level", &["none", "second_factor"]),
                Check::After("expires_at", "created_at"),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "api_token",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("name", Type::Text),
                Column::new("prefix", Type::Text),
                Column::new("hash", Type::Bytes),
                Column::new("created_at", Type::Timestamptz),
                Column::new("expires_at", Type::Timestamptz),
                Column::new("last_used_at", Type::Timestamptz).nullable(),
                Column::new("revoked_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["prefix"]],
            foreign_keys: &[OWNER],
            checks: &[Check::After("expires_at", "created_at")],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "api_token_scope",
            columns: &[
                Column::new("token_id", Type::Uuid),
                Column::new("permission_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
            ],
            primary_key: &["token_id", "permission_code", "scope_id"],
            foreign_keys: &[
                local(&["token_id"], "api_token", OnDelete::Cascade),
                ForeignKey {
                    columns: &["permission_code"],
                    module: Some("access"),
                    table: "permission",
                    references: &["code"],
                    on_delete: OnDelete::Restrict,
                },
                access(&["scope_id"], "scope", OnDelete::Cascade),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "sign_in_attempt",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("campus_id", Type::Uuid),
                Column::new("account_id", Type::Uuid).nullable(),
                Column::new("name_tried_hash", Type::Bytes),
                Column::new("address", Type::Text).nullable(),
                Column::new("outcome", Type::Text),
                Column::new("at", Type::Timestamptz),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                access(&["campus_id"], "scope", OnDelete::Cascade),
                local(&["account_id"], "account", OnDelete::SetNull),
            ],
            checks: &[Check::OneOf("outcome", OUTCOMES)],
            indexes: &[
                Index {
                    columns: &["account_id", "at"],
                    unique: false,
                },
                Index {
                    columns: &["campus_id", "name_tried_hash", "at"],
                    unique: false,
                },
                Index {
                    columns: &["campus_id", "address", "at"],
                    unique: false,
                },
                Index {
                    columns: &["at"],
                    unique: false,
                },
            ],
            without_natural_key: Some("an append-only log for throttling, purged after its retention"),
            ..Table::EMPTY
        }),
    ],
}];
