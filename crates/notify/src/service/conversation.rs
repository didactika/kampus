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

//! Conversations: a direct one exists once per pair of people, a group one gathers the people its creator may
//! write to, and the one of a group of a course follows the members of the group. Who may write to whom is a
//! rule of the campus, and a block between two people stops it either way.

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Cursor, Error, FieldError, Page, PageRequest, Result, Timestamp};
use kampus_courses::{Courses, GroupId};
use serde::Deserialize;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::person;
use crate::permissions::MESSAGE_ANYONE;
use crate::{Conversation, ConversationId, Notify, sql};

/// The most people of a group conversation, its creator included.
pub const MAX_MEMBERS: usize = 50;

/// The longest title of a group conversation.
const MAX_TITLE: usize = 200;

/// What a person asks to open.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Opening {
    /// The conversation with one other person.
    Direct {
        /// The other person.
        account: uuid::Uuid,
    },
    /// A new conversation with several people.
    Group {
        /// Its title.
        title: String,
        /// The other people.
        members: Vec<uuid::Uuid>,
    },
    /// The conversation of a group of a course the person belongs to.
    CourseGroup {
        /// The group.
        group: uuid::Uuid,
    },
}

/// A conversation as the membership of a person sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Membership {
    /// The conversation.
    pub(crate) id: ConversationId,
    /// Its kind.
    pub(crate) kind: String,
    /// Its campus.
    pub(crate) campus: ScopeId,
}

/// The refusal of a conversation or message that is not allowed, which never says who blocked whom.
pub(crate) fn not_allowed() -> Error {
    Error::permission_denied("notify.message_not_allowed")
}

/// A conversation the person does not take part in, told as one that does not exist.
pub(crate) fn not_found() -> Error {
    Error::not_found("notify.conversation_not_found")
}

/// The key of the direct conversation of two people: their ids in order.
fn direct_key(one: PrincipalId, other: PrincipalId) -> String {
    let (low, high) = if one.as_uuid() <= other.as_uuid() {
        (one, other)
    } else {
        (other, one)
    };
    format!("{}:{}", low.as_uuid(), high.as_uuid())
}

/// The cursor of an item in a list: its time and its id.
pub(crate) fn cursor(at: time::OffsetDateTime, id: uuid::Uuid) -> Cursor {
    let micros = i64::try_from(at.unix_timestamp_nanos() / 1000).unwrap_or(i64::MAX);
    let mut bytes = micros.to_be_bytes().to_vec();
    bytes.extend_from_slice(id.as_bytes());
    Cursor::from_bytes(bytes)
}

/// The position a cursor holds.
pub(crate) fn position(
    cursor: Option<&Cursor>,
) -> Result<(Option<time::OffsetDateTime>, Option<uuid::Uuid>)> {
    let Some(cursor) = cursor else {
        return Ok((None, None));
    };
    let invalid = || Error::validation("common.page_invalid", vec![FieldError::new("after", "cursor")]);
    let bytes = cursor.as_bytes();
    if bytes.len() != 24 {
        return Err(invalid());
    }
    let micros = i64::from_be_bytes(bytes[..8].try_into().map_err(|_| invalid())?);
    let at =
        time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1000).map_err(|_| invalid())?;
    let id = uuid::Uuid::from_slice(&bytes[8..]).map_err(|_| invalid())?;
    Ok((Some(at), Some(id)))
}

impl Notify {
    /// Courses, which conversations of groups and the rule of shared courses read.
    fn courses(&self) -> Result<&Courses> {
        self.courses
            .as_ref()
            .ok_or_else(|| Error::unavailable("notify.courses_unavailable"))
    }

    /// Whether a block stands between two people, either way.
    pub(crate) async fn blocked_between(&self, one: PrincipalId, other: PrincipalId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where (account_id = $1 and blocked_id = $2)
                 or (account_id = $2 and blocked_id = $1))",
            self.table("block")
        )))
        .bind(one.as_uuid())
        .bind(other.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    /// Whether the actor may write to another person: both of one campus, no block between them, and the rule
    /// of the campus, which `notify.message.anyone` overrides.
    pub(crate) async fn may_write(&self, actor: &Actor, to: PrincipalId) -> Result<bool> {
        let from = person(actor)?;
        if from == to {
            return Ok(false);
        }
        let (Some(campus), Some(theirs)) = (self.campus_of(from).await?, self.campus_of(to).await?) else {
            return Ok(false);
        };
        if campus != theirs || self.blocked_between(from, to).await? {
            return Ok(false);
        }
        if self.access().check(actor, MESSAGE_ANYONE, campus).await?.allowed {
            return Ok(true);
        }
        match self.messaging(campus).await?.as_str() {
            "campus" => Ok(true),
            "course" => self.courses()?.share_a_course(from, to).await,
            _ => Ok(false),
        }
    }

    /// Opens a conversation: the direct one with a person, found or created once; a new group one; or the one
    /// of a group of a course.
    ///
    /// # Errors
    ///
    /// `permission_denied` (`notify.message_not_allowed`) when the campus rule or a block does not allow it,
    /// `validation` (`notify.conversation_invalid`) for a group without a title or with too many people,
    /// `not_found` for an unknown group.
    pub async fn open_conversation(&self, actor: &Actor, opening: Opening) -> Result<Conversation> {
        let me = person(actor)?;
        let campus = self.campus_of(me).await?.ok_or_else(not_allowed)?;
        let id = match opening {
            Opening::Direct { account } => {
                let other = PrincipalId::from_uuid(account);
                if !self.may_write(actor, other).await? {
                    return Err(not_allowed());
                }
                self.direct(campus, me, other).await?
            }
            Opening::Group { title, members } => self.group(actor, campus, &title, members).await?,
            Opening::CourseGroup { group } => {
                let group = GroupId::from_uuid(group);
                let members = self.courses()?.group_members_of(group).await?;
                if !members.contains(&me) {
                    return Err(not_allowed());
                }
                self.course_group(campus, group).await?
            }
        };
        self.conversation(actor, id).await
    }

    /// The direct conversation of two people, created once whoever asks first.
    async fn direct(&self, campus: ScopeId, me: PrincipalId, other: PrincipalId) -> Result<ConversationId> {
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, kind, direct_key, created_by) values ($1, $2, 'direct', $3, $4)
             on conflict (direct_key) do nothing",
            self.table("conversation")
        )))
        .bind(ConversationId::generate().as_uuid())
        .bind(campus.as_uuid())
        .bind(direct_key(me, other))
        .bind(me.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select id from {} where direct_key = $1",
            self.table("conversation")
        )))
        .bind(direct_key(me, other))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for account in [me, other] {
            sqlx::query(sql(format!(
                "insert into {} (conversation_id, account_id) values ($1, $2) on conflict do nothing",
                self.table("member")
            )))
            .bind(id)
            .bind(account.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(ConversationId::from_uuid(id))
    }

    /// A new group conversation with the people the actor may write to.
    async fn group(
        &self,
        actor: &Actor,
        campus: ScopeId,
        title: &str,
        members: Vec<uuid::Uuid>,
    ) -> Result<ConversationId> {
        let me = person(actor)?;
        let title = title.trim();
        if title.is_empty() || title.chars().count() > MAX_TITLE {
            return Err(Error::validation(
                "notify.conversation_invalid",
                vec![FieldError::new("title", "length")],
            ));
        }
        let mut others: Vec<PrincipalId> = members.into_iter().map(PrincipalId::from_uuid).collect();
        others.sort_by_key(PrincipalId::as_uuid);
        others.dedup();
        others.retain(|p| *p != me);
        if others.is_empty() || others.len() >= MAX_MEMBERS {
            return Err(Error::validation(
                "notify.conversation_invalid",
                vec![FieldError::new("members", "count")],
            ));
        }
        for other in &others {
            if !self.may_write(actor, *other).await? {
                return Err(not_allowed());
            }
        }
        let id = ConversationId::generate();
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, kind, title, created_by) values ($1, $2, 'group', $3, $4)",
            self.table("conversation")
        )))
        .bind(id.as_uuid())
        .bind(campus.as_uuid())
        .bind(title)
        .bind(me.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for account in std::iter::once(me).chain(others) {
            sqlx::query(sql(format!(
                "insert into {} (conversation_id, account_id) values ($1, $2)",
                self.table("member")
            )))
            .bind(id.as_uuid())
            .bind(account.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// The conversation of a group of a course, created once, with the members of the group now.
    async fn course_group(&self, campus: ScopeId, group: GroupId) -> Result<ConversationId> {
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, kind, course_group_id) values ($1, $2, 'course_group', $3)
             on conflict (course_group_id) do nothing",
            self.table("conversation")
        )))
        .bind(ConversationId::generate().as_uuid())
        .bind(campus.as_uuid())
        .bind(group.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select id from {} where course_group_id = $1",
            self.table("conversation")
        )))
        .bind(group.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let id = ConversationId::from_uuid(id);
        self.follow_group(id, group).await?;
        Ok(id)
    }

    /// Makes the members of the conversation of a group those of the group now: new ones join, those who left
    /// the group leave it.
    async fn follow_group(&self, id: ConversationId, group: GroupId) -> Result<()> {
        let members: Vec<uuid::Uuid> = self
            .courses()?
            .group_members_of(group)
            .await?
            .iter()
            .map(PrincipalId::as_uuid)
            .collect();
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {m} (conversation_id, account_id) select $1, unnest($2::uuid[])
             on conflict (conversation_id, account_id) do update set left_at = null, joined_at =
                 case when {m}.left_at is null then {m}.joined_at else now() end",
            m = self.table("member")
        )))
        .bind(id.as_uuid())
        .bind(&members)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set left_at = now() where conversation_id = $1 and left_at is null
                 and not (account_id = any($2))",
            self.table("member")
        )))
        .bind(id.as_uuid())
        .bind(&members)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The conversation of a person who takes part in it now; the conversation of a group follows the group
    /// first.
    pub(crate) async fn membership(&self, me: PrincipalId, id: ConversationId) -> Result<Membership> {
        let row = sqlx::query(sql(format!(
            "select kind, campus_id, course_group_id from {} where id = $1",
            self.table("conversation")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(not_found)?;
        if let Some(group) = row.get::<Option<uuid::Uuid>, _>("course_group_id") {
            self.follow_group(id, GroupId::from_uuid(group)).await?;
        }
        let member: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where conversation_id = $1 and account_id = $2 and left_at is null)",
            self.table("member")
        )))
        .bind(id.as_uuid())
        .bind(me.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if !member {
            return Err(not_found());
        }
        Ok(Membership {
            id,
            kind: row.get("kind"),
            campus: ScopeId::from_uuid(row.get("campus_id")),
        })
    }

    /// The people who take part in a conversation now.
    pub(crate) async fn members_of(&self, id: ConversationId) -> Result<Vec<(PrincipalId, bool)>> {
        let rows = sqlx::query(sql(format!(
            "select account_id, muted from {} where conversation_id = $1 and left_at is null order by joined_at, account_id",
            self.table("member")
        )))
        .bind(id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (PrincipalId::from_uuid(r.get("account_id")), r.get("muted")))
            .collect())
    }

    /// The query of the conversations of a person, with what each shows them.
    fn conversations_query(&self, one: bool) -> String {
        format!(
            "with seen as (
                 select c.id, c.kind, c.title, c.course_group_id, c.created_at, m.muted, m.last_read_at,
                     (select max(x.sent_at) from {message} x where x.conversation_id = c.id and not exists
                         (select 1 from {deletion} d where d.message_id = x.id and d.account_id = $1)) as last_at
                 from {conversation} c join {member} m on m.conversation_id = c.id
                 where m.account_id = $1 and m.left_at is null {only})
             select s.*, coalesce(s.last_at, s.created_at) as activity,
                 (select count(*) from {message} x where x.conversation_id = s.id and x.author_id <> $1
                     and x.sent_at > coalesce(s.last_read_at, '-infinity') and not exists
                     (select 1 from {deletion} d where d.message_id = x.id and d.account_id = $1)) as unread,
                 (select coalesce(array_agg(account_id order by joined_at, account_id), '{{}}') from {member}
                     where conversation_id = s.id and left_at is null) as members
             from seen s",
            message = self.table("message"),
            deletion = self.table("message_deletion"),
            conversation = self.table("conversation"),
            member = self.table("member"),
            only = if one { "and c.id = $2" } else { "" },
        )
    }

    /// A conversation from its row.
    fn conversation_of(row: &PgRow) -> Conversation {
        Conversation {
            id: ConversationId::from_uuid(row.get("id")),
            kind: row.get("kind"),
            title: row.get("title"),
            course_group: row.get("course_group_id"),
            members: row
                .get::<Vec<uuid::Uuid>, _>("members")
                .into_iter()
                .map(PrincipalId::from_uuid)
                .collect(),
            last_message_at: row
                .get::<Option<time::OffsetDateTime>, _>("last_at")
                .map(Timestamp::from_offset_date_time),
            unread: row.get("unread"),
            muted: row.get("muted"),
        }
    }

    /// A conversation the actor takes part in.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.conversation_not_found`) for one they do not take part in.
    pub async fn conversation(&self, actor: &Actor, id: ConversationId) -> Result<Conversation> {
        let me = person(actor)?;
        self.membership(me, id).await?;
        let row = sqlx::query(sql(self.conversations_query(true)))
            .bind(me.as_uuid())
            .bind(id.as_uuid())
            .fetch_optional(self.db.pool())
            .await
            .map_err(kampus_db::classify)?
            .ok_or_else(not_found)?;
        Ok(Self::conversation_of(&row))
    }

    /// The conversations of the actor, the most recently active first.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor, `validation` for a cursor that is not one.
    pub async fn conversations(&self, actor: &Actor, page: &PageRequest) -> Result<Page<Conversation>> {
        let me = person(actor)?;
        let (at, id) = position(page.after())?;
        let rows = sqlx::query(sql(format!(
            "select * from ({}) listed where ($2::timestamptz is null or (activity, id) < ($2, $3))
             order by activity desc, id desc limit $4",
            self.conversations_query(false)
        )))
        .bind(me.as_uuid())
        .bind(at)
        .bind(id)
        .bind(i64::from(page.limit()) + 1)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let limit = usize::try_from(page.limit()).unwrap_or(usize::MAX);
        let next = (rows.len() > limit).then(|| {
            let last = &rows[limit - 1];
            cursor(last.get("activity"), last.get("id"))
        });
        Ok(Page {
            items: rows.iter().take(limit).map(Self::conversation_of).collect(),
            next,
        })
    }

    /// Marks every message of a conversation read for the actor.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.conversation_not_found`) for one they do not take part in.
    pub async fn mark_conversation_read(&self, actor: &Actor, id: ConversationId) -> Result<()> {
        self.set_member(actor, id, "last_read_at = now()").await
    }

    /// Turns the notifications of a conversation off or on for the actor.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.conversation_not_found`) for one they do not take part in.
    pub async fn mute(&self, actor: &Actor, id: ConversationId, muted: bool) -> Result<()> {
        self.set_member(actor, id, if muted { "muted = true" } else { "muted = false" })
            .await
    }

    /// Leaves a group conversation; a direct one stays, and the one of a group follows the group.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.conversation_not_found`) for one they do not take part in, `validation`
    /// (`notify.conversation_leave_invalid`) for one that is not a group conversation.
    pub async fn leave(&self, actor: &Actor, id: ConversationId) -> Result<()> {
        let found = self.membership(person(actor)?, id).await?;
        if found.kind != "group" {
            return Err(Error::validation(
                "notify.conversation_leave_invalid",
                vec![FieldError::new("kind", "group")],
            ));
        }
        self.set_member(actor, id, "left_at = now()").await
    }

    /// Changes the membership of the actor in a conversation.
    async fn set_member(&self, actor: &Actor, id: ConversationId, change: &'static str) -> Result<()> {
        let me = person(actor)?;
        self.membership(me, id).await?;
        sqlx::query(sql(format!(
            "update {} set {change} where conversation_id = $1 and account_id = $2",
            self.table("member")
        )))
        .bind(id.as_uuid())
        .bind(me.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    }
}
