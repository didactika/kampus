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

//! Messages: written in sanitised rich text, with files attached, by someone who takes part in a conversation
//! and may still write to the other person of a direct one; the other members are notified. Deleting removes a message for the
//! person who deletes it, and a job purges messages every member deleted or the campus keeps no longer.

use kampus_access::{Actor, PrincipalId};
use kampus_common::text::{plain_of_rich, preview, sanitise_rich};
use kampus_common::{Error, FieldError, Page, PageRequest, Result, Timestamp};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::conversation::{cursor, not_allowed, position};
use super::person;
use crate::settings::files::MESSAGE_AREA;
use crate::settings::notifications::MESSAGE_RECEIVED;
use crate::{ConversationId, Message, MessageId, Notify, sql};

/// The longest message, in bytes of its sanitised rich text.
pub const MAX_BODY: usize = 20_000;

/// The characters of a message its notification shows.
const PREVIEW: usize = 140;

/// The most files of one message.
pub const MAX_FILES: usize = 10;

/// The columns of a message, with its files in order.
const fn columns() -> &'static str {
    "m.id, m.conversation_id, m.author_id, m.body, m.sent_at, coalesce((select array_agg(f.file_id order by f.position)
         from {files} f where f.message_id = m.id), '{}') as files"
}

/// How many messages one statement of the purge removes.
const PURGE_BATCH: i64 = 1000;

/// A message from its row.
fn message(row: &PgRow) -> Message {
    Message {
        id: MessageId::from_uuid(row.get("id")),
        conversation: ConversationId::from_uuid(row.get("conversation_id")),
        author: PrincipalId::from_uuid(row.get("author_id")),
        body: row.get("body"),
        files: row.get("files"),
        sent_at: Timestamp::from_offset_date_time(row.get("sent_at")),
    }
}

impl Notify {
    /// Sends a message to a conversation, with files the actor uploaded to the area of messages, and notifies
    /// the other members who did not mute it.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.conversation_not_found`) for a conversation the actor does not take part in,
    /// `permission_denied` (`notify.message_not_allowed`) when they may no longer write to the other person
    /// of a direct conversation, `validation` (`notify.message_invalid`) for a message without text or files,
    /// too long or with too many files, and the errors of attaching a file.
    pub async fn send_message(
        &self,
        actor: &Actor,
        conversation: ConversationId,
        body: &str,
        files: &[kampus_files::FileId],
    ) -> Result<Message> {
        let me = person(actor)?;
        let found = self.membership(me, conversation).await?;
        let members = self.members_of(conversation).await?;
        if found.kind == "direct" {
            for (other, _) in members.iter().filter(|(p, _)| *p != me) {
                if !self.may_write(actor, *other).await? {
                    return Err(not_allowed());
                }
            }
        }
        let body = sanitise_rich(body.trim());
        let plain = plain_of_rich(&body);
        let wrong = if plain.is_empty() && files.is_empty() {
            Some(("body", "required"))
        } else if body.len() > MAX_BODY {
            Some(("body", "max_length"))
        } else if files.len() > MAX_FILES {
            Some(("files", "max_items"))
        } else {
            None
        };
        if let Some((field, rule)) = wrong {
            return Err(Error::validation(
                "notify.message_invalid",
                vec![FieldError::new(field, rule)],
            ));
        }
        let attachments = match (files.is_empty(), self.files.as_ref()) {
            (true, _) => None,
            (false, Some(store)) => Some(store),
            (false, None) => return Err(Error::unavailable("notify.files_unavailable")),
        };
        let id = MessageId::generate();
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, conversation_id, author_id, body) values ($1, $2, $3, $4)
             returning id, conversation_id, author_id, body, sent_at, '{{}}'::uuid[] as files",
            self.table("message")
        )))
        .bind(id.as_uuid())
        .bind(conversation.as_uuid())
        .bind(me.as_uuid())
        .bind(&body)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set last_read_at = $3 where conversation_id = $1 and account_id = $2",
            self.table("member")
        )))
        .bind(conversation.as_uuid())
        .bind(me.as_uuid())
        .bind(row.get::<time::OffsetDateTime, _>("sent_at"))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(store) = attachments {
            for (position, file) in files.iter().enumerate() {
                store
                    .attach_in(&mut tx, *file, MESSAGE_AREA, Some(me.as_uuid()))
                    .await?;
                sqlx::query(sql(format!(
                    "insert into {} (message_id, file_id, position) values ($1, $2, $3)",
                    self.table("message_file")
                )))
                .bind(id.as_uuid())
                .bind(file.as_uuid())
                .bind(i32::try_from(position).unwrap_or(i32::MAX))
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        let sent = Message {
            files: files.iter().map(kampus_files::FileId::as_uuid).collect(),
            ..message(&row)
        };
        let payload = json!({
            "conversation": conversation.as_uuid(),
            "message": id.as_uuid(),
            "author": me.as_uuid(),
            "preview": preview(&plain, PREVIEW),
        });
        for (member, muted) in members {
            if member != me && !muted {
                self.notify(
                    MESSAGE_RECEIVED,
                    member,
                    payload.clone(),
                    Some(&id.as_uuid().to_string()),
                )
                .await?;
            }
        }
        Ok(sent)
    }

    /// The messages of a conversation the actor did not delete, newest first.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.conversation_not_found`) for a conversation the actor does not take part in,
    /// `validation` for a cursor that is not one.
    pub async fn messages(
        &self,
        actor: &Actor,
        conversation: ConversationId,
        page: &PageRequest,
    ) -> Result<Page<Message>> {
        let me = person(actor)?;
        self.membership(me, conversation).await?;
        let (at, after) = position(page.after())?;
        let rows = sqlx::query(sql(format!(
            "select {} from {} m
             where m.conversation_id = $1 and not exists
                 (select 1 from {} d where d.message_id = m.id and d.account_id = $2)
               and ($3::timestamptz is null or (m.sent_at, m.id) < ($3, $4))
             order by m.sent_at desc, m.id desc limit $5",
            columns().replace("{files}", &self.table("message_file")),
            self.table("message"),
            self.table("message_deletion")
        )))
        .bind(conversation.as_uuid())
        .bind(me.as_uuid())
        .bind(at)
        .bind(after)
        .bind(i64::from(page.limit()) + 1)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let limit = usize::try_from(page.limit()).unwrap_or(usize::MAX);
        let next = (rows.len() > limit).then(|| {
            let last = &rows[limit - 1];
            cursor(last.get("sent_at"), last.get("id"))
        });
        Ok(Page {
            items: rows.iter().take(limit).map(message).collect(),
            next,
        })
    }

    /// Deletes a message for the actor; it stays for the other members until they delete it too.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.message_not_found`) for a message of a conversation the actor never took part in.
    pub async fn delete_message(&self, actor: &Actor, id: MessageId) -> Result<()> {
        let me = person(actor)?;
        let deleted = sqlx::query(sql(format!(
            "insert into {d} (message_id, account_id)
             select m.id, $2 from {m} m join {mb} mb on mb.conversation_id = m.conversation_id and mb.account_id = $2
             where m.id = $1
             on conflict do nothing",
            d = self.table("message_deletion"),
            m = self.table("message"),
            mb = self.table("member")
        )))
        .bind(id.as_uuid())
        .bind(me.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if deleted == 0 {
            let theirs: bool = sqlx::query_scalar(sql(format!(
                "select exists (select 1 from {} m join {} mb on mb.conversation_id = m.conversation_id
                     and mb.account_id = $2 where m.id = $1)",
                self.table("message"),
                self.table("member")
            )))
            .bind(id.as_uuid())
            .bind(me.as_uuid())
            .fetch_one(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            if !theirs {
                return Err(Error::not_found("notify.message_not_found"));
            }
        }
        Ok(())
    }

    /// Purges the messages every member deleted, and those older than their campus keeps; returns how many.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn purge_messages(&self) -> Result<u64> {
        let mut purged = 0;
        loop {
            let due: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
                "select x.id from {m} x where not exists
                     (select 1 from {mb} mb where mb.conversation_id = x.conversation_id and not exists
                         (select 1 from {d} d where d.message_id = x.id and d.account_id = mb.account_id))
                 limit $1",
                m = self.table("message"),
                mb = self.table("member"),
                d = self.table("message_deletion")
            )))
            .bind(PURGE_BATCH)
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            if due.is_empty() {
                break;
            }
            purged += self.remove_messages(&due).await?;
        }
        let campuses: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct campus_id from {}",
            self.table("conversation")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        for campus in campuses {
            let days = self
                .retention_days(kampus_access::ScopeId::from_uuid(campus))
                .await?;
            if days == 0 {
                continue;
            }
            let old: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
                "select m.id from {} m join {} c on c.id = m.conversation_id where c.campus_id = $1
                     and m.sent_at < now() - make_interval(days => $2)",
                self.table("message"),
                self.table("conversation")
            )))
            .bind(campus)
            .bind(i32::try_from(days).unwrap_or(i32::MAX))
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            for chunk in old.chunks(usize::try_from(PURGE_BATCH).unwrap_or(1000)) {
                purged += self.remove_messages(chunk).await?;
            }
        }
        Ok(purged)
    }

    /// Removes messages with their files, whose content goes to the trash when nothing else uses it.
    pub(crate) async fn remove_messages(&self, ids: &[uuid::Uuid]) -> Result<u64> {
        let mut tx = self.db.begin().await?;
        let files: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "delete from {} where message_id = any($1) returning file_id",
            self.table("message_file")
        )))
        .bind(ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(store) = self.files.as_ref() {
            for file in files {
                match store
                    .detach_in(&mut tx, kampus_files::FileId::from_uuid(file))
                    .await
                {
                    Err(error) if error.kind() != kampus_common::ErrorKind::NotFound => return Err(error),
                    _ => {}
                }
            }
        }
        let removed = sqlx::query(sql(format!(
            "delete from {} where id = any($1)",
            self.table("message")
        )))
        .bind(ids)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(removed)
    }
}
