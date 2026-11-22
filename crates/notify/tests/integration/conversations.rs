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

//! Conversations: a direct one exists once per pair, the one of a group of a course follows the group,
//! messages are sanitised and notify the others, a block or the rule of the campus stops writing, and a
//! message every member deleted is purged.

use kampus_access::Actor;
use kampus_common::{ErrorKind, PageRequest};
use kampus_courses::{GroupFields, GroupVisibility};
use kampus_notify::service::conversation::Opening;
use kampus_notify::{Conversation, ConversationId};
use serde_json::json;

use crate::world::{World, who, world};

/// The direct conversation of an actor with another.
async fn direct(w: &World, from: &Actor, to: &Actor) -> kampus_common::Result<Conversation> {
    w.notify
        .open_conversation(
            from,
            Opening::Direct {
                account: who(to).as_uuid(),
            },
        )
        .await
}

/// The texts of the messages an actor sees in a conversation, oldest first.
async fn texts(w: &World, actor: &Actor, conversation: ConversationId) -> Vec<String> {
    let mut found: Vec<String> = w
        .notify
        .messages(actor, conversation, &PageRequest::default())
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|m| m.body)
        .collect();
    found.reverse();
    found
}

/// Sets a setting of messaging in the campus.
async fn set(w: &World, key: &str, value: serde_json::Value) {
    kampus_settings::Settings::new(
        w.db.clone(),
        kampus_access::Access::new(w.db.clone(), std::sync::Arc::new(kampus_common::SystemClock)),
    )
    .set(&w.admin, key, w.campus, value, false, None)
    .await
    .unwrap();
}

#[tokio::test]
async fn a_direct_conversation_exists_once_and_its_messages_are_sanitised_and_notified() {
    let w = world().await;
    let opened = direct(&w, &w.lea, &w.max).await.unwrap();
    let again = direct(&w, &w.max, &w.lea).await.unwrap();
    assert_eq!(opened.id, again.id, "one conversation per pair, whoever opens it");
    assert_eq!(opened.kind, "direct");
    assert_eq!(opened.members.len(), 2);

    let sent = w
        .notify
        .send_message(
            &w.lea,
            opened.id,
            "<p>Hi <b>Max</b><script>alert(1)</script></p>",
            &[],
        )
        .await
        .unwrap();
    assert_eq!(sent.body, "<p>Hi Max</p>", "only the elements of rich text stay");
    assert_eq!(
        w.notify
            .send_message(&w.lea, opened.id, "<p> </p><script>x</script>", &[])
            .await
            .unwrap_err()
            .code(),
        "notify.message_invalid"
    );
    let inbox = w.notify.inbox(&w.max, false, 10, None).await.unwrap();
    assert_eq!(inbox.notifications.len(), 1);
    assert_eq!(inbox.notifications[0].type_code, "notify.message_received");
    assert_eq!(inbox.notifications[0].payload["preview"], "Hi Max");
    assert_eq!(
        w.notify
            .inbox(&w.lea, false, 10, None)
            .await
            .unwrap()
            .notifications
            .len(),
        0,
        "the author is not notified"
    );

    let seen = w.notify.conversation(&w.max, opened.id).await.unwrap();
    assert_eq!(seen.unread, 1);
    w.notify.mark_conversation_read(&w.max, opened.id).await.unwrap();
    assert_eq!(w.notify.conversation(&w.max, opened.id).await.unwrap().unread, 0);
    assert_eq!(
        w.notify.conversation(&w.lea, opened.id).await.unwrap().unread,
        0,
        "the author read it"
    );

    w.notify.mute(&w.max, opened.id, true).await.unwrap();
    w.notify
        .send_message(&w.lea, opened.id, "Still there?", &[])
        .await
        .unwrap();
    assert_eq!(
        w.notify
            .inbox(&w.max, false, 10, None)
            .await
            .unwrap()
            .notifications
            .len(),
        1,
        "a muted conversation notifies nobody"
    );
    assert_eq!(
        texts(&w, &w.max, opened.id).await,
        ["<p>Hi Max</p>", "Still there?"]
    );
    let listed = w
        .notify
        .conversations(&w.max, &PageRequest::default())
        .await
        .unwrap();
    assert_eq!(listed.items.len(), 1);
    assert!(listed.items[0].muted);
    assert_eq!(
        w.notify
            .conversation(&w.admin, opened.id)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound,
        "nobody else sees it"
    );
    assert_eq!(
        w.notify.leave(&w.lea, opened.id).await.unwrap_err().code(),
        "notify.conversation_leave_invalid"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_block_or_the_rule_of_the_campus_stops_writing() {
    let w = world().await;
    let opened = direct(&w, &w.lea, &w.max).await.unwrap();
    w.notify.block(&w.max, who(&w.lea)).await.unwrap();
    for refused in [
        w.notify
            .send_message(&w.lea, opened.id, "Hello", &[])
            .await
            .unwrap_err(),
        w.notify
            .send_message(&w.max, opened.id, "Hello", &[])
            .await
            .unwrap_err(),
    ] {
        assert_eq!(
            refused.code(),
            "notify.message_not_allowed",
            "either way, without saying who"
        );
    }
    assert_eq!(w.notify.blocked(&w.max).await.unwrap(), [who(&w.lea)]);
    w.notify.unblock(&w.max, who(&w.lea)).await.unwrap();
    w.notify
        .send_message(&w.lea, opened.id, "Hello", &[])
        .await
        .unwrap();
    assert_eq!(
        w.notify.block(&w.lea, who(&w.lea)).await.unwrap_err().code(),
        "notify.block_invalid"
    );

    set(&w, "notify.messaging", json!("nobody")).await;
    assert_eq!(
        direct(&w, &w.lea, &w.max).await.unwrap_err().code(),
        "notify.message_not_allowed"
    );
    assert_eq!(
        w.notify
            .send_message(&w.lea, opened.id, "Hello", &[])
            .await
            .unwrap_err()
            .code(),
        "notify.message_not_allowed"
    );
    direct(&w, &w.admin, &w.max)
        .await
        .expect("whoever may write to anyone of the campus still writes");
    set(&w, "notify.messaging", json!("campus")).await;
    w.notify
        .send_message(&w.lea, opened.id, "Hello again", &[])
        .await
        .unwrap();
    w.drop().await;
}

#[tokio::test]
async fn the_conversation_of_a_group_follows_its_members() {
    let w = world().await;
    let group = w
        .courses
        .create_group(
            &w.admin,
            w.course.id,
            GroupFields {
                name: "Lab".to_owned(),
                code: None,
                description: None,
                section: None,
                visibility: GroupVisibility::Everyone,
            },
        )
        .await
        .unwrap();
    let opening = Opening::CourseGroup {
        group: group.id.as_uuid(),
    };
    assert_eq!(
        w.notify
            .open_conversation(&w.lea, opening.clone())
            .await
            .unwrap_err()
            .code(),
        "notify.message_not_allowed",
        "only its members"
    );
    for member in [&w.lea, &w.max] {
        w.courses
            .add_member(&w.admin, group.id, who(member))
            .await
            .unwrap();
    }
    let opened = w.notify.open_conversation(&w.lea, opening.clone()).await.unwrap();
    assert_eq!(opened.kind, "course_group");
    assert_eq!(opened.members.len(), 2);
    assert_eq!(
        w.notify.open_conversation(&w.max, opening).await.unwrap().id,
        opened.id
    );
    w.notify
        .send_message(&w.max, opened.id, "Lab at five", &[])
        .await
        .unwrap();

    w.courses
        .remove_member(&w.admin, group.id, who(&w.max))
        .await
        .unwrap();
    assert_eq!(
        w.notify
            .messages(&w.max, opened.id, &PageRequest::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound,
        "who left the group left its conversation"
    );
    assert_eq!(texts(&w, &w.lea, opened.id).await, ["Lab at five"]);
    assert_eq!(
        w.notify.conversation(&w.lea, opened.id).await.unwrap().members,
        [who(&w.lea)]
    );
    w.drop().await;
}

#[tokio::test]
async fn a_group_conversation_gathers_people_and_a_message_everyone_deleted_is_purged() {
    let w = world().await;
    let opened = w
        .notify
        .open_conversation(
            &w.admin,
            Opening::Group {
                title: " Study ".to_owned(),
                members: vec![
                    who(&w.lea).as_uuid(),
                    who(&w.max).as_uuid(),
                    who(&w.lea).as_uuid(),
                ],
            },
        )
        .await
        .unwrap();
    assert_eq!(opened.title.as_deref(), Some("Study"));
    assert_eq!(opened.members.len(), 3);
    let invalid = w
        .notify
        .open_conversation(
            &w.lea,
            Opening::Group {
                title: String::new(),
                members: vec![who(&w.max).as_uuid()],
            },
        )
        .await
        .unwrap_err();
    assert_eq!(invalid.code(), "notify.conversation_invalid");

    let first = w
        .notify
        .send_message(&w.admin, opened.id, "First", &[])
        .await
        .unwrap();
    w.notify
        .send_message(&w.admin, opened.id, "Second", &[])
        .await
        .unwrap();
    for actor in [&w.admin, &w.lea] {
        w.notify.delete_message(actor, first.id).await.unwrap();
    }
    assert_eq!(texts(&w, &w.lea, opened.id).await, ["Second"]);
    assert_eq!(texts(&w, &w.max, opened.id).await, ["First", "Second"]);
    assert_eq!(w.notify.purge_messages().await.unwrap(), 0, "max still keeps it");
    w.notify.leave(&w.max, opened.id).await.unwrap();
    w.notify.delete_message(&w.max, first.id).await.unwrap();
    assert_eq!(w.notify.purge_messages().await.unwrap(), 1);
    assert_eq!(
        w.notify
            .delete_message(&w.max, first.id)
            .await
            .unwrap_err()
            .code(),
        "notify.message_not_found"
    );

    set(&w, "notify.message_retention_days", json!(1)).await;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "update {}.\"message\" set sent_at = now() - interval '2 days'",
        w.db.prefix().schema("notify")
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    assert_eq!(
        w.notify.purge_messages().await.unwrap(),
        1,
        "older than the campus keeps"
    );
    assert_eq!(texts(&w, &w.lea, opened.id).await.len(), 0);

    let report = w.notify.erase_personal_data(who(&w.lea)).await.unwrap();
    assert_eq!(report["deleted"]["member"], 1);
    w.drop().await;
}

/// A file a person uploaded to the area of messages.
async fn upload(w: &World, actor: &Actor, name: &str) -> kampus_files::File {
    let content: &'static [u8] = b"lab notes";
    let upload = w
        .files
        .open_upload(
            actor,
            ("notify.message", w.campus),
            name,
            i64::try_from(content.len()).unwrap(),
            None,
        )
        .await
        .unwrap();
    w.files
        .send_upload(actor, upload.id, bytes::Bytes::from_static(content))
        .await
        .unwrap();
    w.files.complete_upload(actor, upload.id).await.unwrap()
}

#[tokio::test]
async fn files_of_a_message_are_read_by_its_members_and_go_with_the_message() {
    let w = world().await;
    let services = kampus_module::ServiceContext::default().with(w.notify.clone());
    let opened = direct(&w, &w.lea, &w.max).await.unwrap();
    let notes = upload(&w, &w.lea, "notes.txt").await;
    let theirs = upload(&w, &w.max, "other.txt").await;
    assert_eq!(
        w.notify
            .send_message(&w.lea, opened.id, "", &[theirs.id])
            .await
            .unwrap_err()
            .code(),
        "files.file_not_found",
        "only one's own uploads"
    );
    let sent = w
        .notify
        .send_message(&w.lea, opened.id, "", &[notes.id])
        .await
        .unwrap();
    assert_eq!(sent.files, [notes.id.as_uuid()], "a message may be only a file");
    let listed = w
        .notify
        .messages(&w.max, opened.id, &PageRequest::default())
        .await
        .unwrap();
    assert_eq!(listed.items[0].files, [notes.id.as_uuid()]);
    let read = |actor: &Actor| {
        let principal = Some(who(actor).as_uuid());
        let files = w.files.clone();
        let services = services.clone();
        async move { files.file(principal, notes.id, &services).await }
    };
    assert!(read(&w.max).await.is_ok(), "the other member reads it");
    assert_eq!(read(&w.admin).await.unwrap_err().code(), "files.file_not_found");

    for actor in [&w.lea, &w.max] {
        w.notify.delete_message(actor, sent.id).await.unwrap();
    }
    assert_eq!(w.notify.purge_messages().await.unwrap(), 1);
    assert_eq!(
        read(&w.lea).await.unwrap_err().code(),
        "files.file_not_found",
        "the file went with the message"
    );
    w.drop().await;
}
