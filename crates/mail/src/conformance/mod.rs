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
// @package   kampus-mail
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The conformance suite of the mail port: an adapter is supported only when it passes every check here,
//! run against a real server whose received messages the test can read.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "a conformance failure must stop the test"
)]

use async_trait::async_trait;

use crate::{Mail, Mailer};

/// One message as the server received it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    /// The recipients of its envelope.
    pub to: Vec<String>,
    /// The message as it went over the wire, headers and body.
    pub data: String,
}

/// What the server received, oldest first.
#[async_trait]
pub trait Inbox: Send + Sync {
    /// The messages received so far.
    async fn received(&self) -> Vec<Received>;
}

/// The value of a header of a message, unfolded.
fn header<'a>(data: &'a str, name: &str) -> Option<&'a str> {
    let head = data.split("\r\n\r\n").next()?;
    head.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.eq_ignore_ascii_case(name).then(|| value.trim())
    })
}

/// Runs every check.
pub async fn run(mailer: &dyn Mailer, inbox: &dyn Inbox) {
    mailer.check().await.expect("the server answers");
    let before = inbox.received().await.len();

    let one = Mail::new(
        vec!["lea@example.org".to_owned()],
        "New grade",
        "Your quiz was graded.\nSee you soon.",
    )
    .unwrap();
    mailer.send(&one).await.expect("a message is sent");
    let both = Mail::new(
        vec!["lea@example.org".to_owned(), "max@example.org".to_owned()],
        "Two\r\nBcc: intruder@example.org",
        "Hello",
    )
    .unwrap();
    mailer.send(&both).await.expect("a message to two is sent");
    let accented = Mail::new(vec!["lea@example.org".to_owned()], "Calificación nueva", "Ñandú").unwrap();
    mailer
        .send(&accented)
        .await
        .expect("text that is not ASCII is sent");

    let received = inbox.received().await;
    let received = &received[before..];
    assert_eq!(received.len(), 3, "each message arrives once");
    assert_eq!(received[0].to, ["lea@example.org"]);
    assert_eq!(header(&received[0].data, "Subject"), Some("New grade"));
    assert!(
        header(&received[0].data, "From").is_some(),
        "it says who sends it"
    );
    assert!(header(&received[0].data, "Date").is_some(), "it is dated");
    assert!(
        received[0].data.contains("Your quiz was graded."),
        "its text arrives: {}",
        received[0].data
    );
    assert_eq!(received[1].to, ["lea@example.org", "max@example.org"]);
    assert!(
        header(&received[1].data, "Bcc").is_none() && !received[1].to.iter().any(|t| t.contains("intruder")),
        "a subject never adds a header"
    );
    assert_eq!(
        header(&received[1].data, "Subject"),
        Some("Two Bcc: intruder@example.org")
    );
    assert!(
        header(&received[2].data, "Subject").is_some_and(|s| s.starts_with("=?utf-8?")),
        "a subject that is not ASCII is encoded"
    );
}
