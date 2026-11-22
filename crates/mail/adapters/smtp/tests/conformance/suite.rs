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
// @package   kampus-mail-smtp
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The mail conformance suite against an SMTP server that speaks the protocol and keeps what it receives,
//! and against a real one when `KAMPUS_TEST_SMTP_URL` names it with the web interface of Mailpit in
//! `KAMPUS_TEST_MAILPIT_URL`.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kampus_common::AdapterSettings;
use kampus_mail::conformance::{self, Inbox, Received};
use kampus_mail::{Mail, Mailer};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// What the fake server received, and the address it refuses for good.
#[derive(Debug, Default, Clone)]
struct Kept(Arc<Mutex<Vec<Received>>>);

#[async_trait]
impl Inbox for Kept {
    async fn received(&self) -> Vec<Received> {
        self.0.lock().unwrap().clone()
    }
}

/// The address a mailbox of the fake server does not exist for.
const UNKNOWN: &str = "gone@example.org";

/// Answers one connection as an SMTP server would.
async fn answer(stream: TcpStream, kept: Kept) {
    let (read, mut write) = stream.into_split();
    let mut lines = BufReader::new(read).lines();
    put(&mut write, b"220 kampus.test ESMTP\r\n").await;
    let mut to = Vec::new();
    while let Ok(Some(line)) = lines.next_line().await {
        let upper = line.to_ascii_uppercase();
        let reply: &[u8] = if upper.starts_with("EHLO") || upper.starts_with("HELO") {
            b"250-kampus.test\r\n250 8BITMIME\r\n"
        } else if upper.starts_with("MAIL FROM") || upper.starts_with("RSET") || upper.starts_with("NOOP") {
            to.clear();
            b"250 OK\r\n"
        } else if upper.starts_with("RCPT TO") {
            if line.contains(UNKNOWN) {
                b"550 no such mailbox\r\n"
            } else {
                let address = line.split(['<', '>']).nth(1).unwrap_or_default().to_owned();
                to.push(address);
                b"250 OK\r\n"
            }
        } else if upper.starts_with("DATA") {
            put(&mut write, b"354 go on\r\n").await;
            let mut data = String::new();
            while let Ok(Some(line)) = lines.next_line().await {
                if line == "." {
                    break;
                }
                data.push_str(&line);
                data.push_str("\r\n");
            }
            kept.0.lock().unwrap().push(Received {
                to: std::mem::take(&mut to),
                data,
            });
            b"250 queued\r\n"
        } else if upper.starts_with("QUIT") {
            put(&mut write, b"221 bye\r\n").await;
            return;
        } else {
            b"502 not here\r\n"
        };
        put(&mut write, reply).await;
    }
}

/// Writes a reply, ignoring a client that went away.
async fn put(write: &mut tokio::net::tcp::OwnedWriteHalf, bytes: &[u8]) {
    let _ = write.write_all(bytes).await;
}

/// An SMTP server on a free local port; returns its address and what it keeps.
async fn fake_server() -> (String, Kept) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let kept = Kept::default();
    let keeping = kept.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(answer(stream, keeping.clone()));
        }
    });
    (address, kept)
}

/// The settings of the adapter on a URL.
fn settings(url: String) -> AdapterSettings {
    AdapterSettings {
        url,
        options: [("from".to_owned(), "Kampus <no-reply@kampus.test>".to_owned())].into(),
        ..AdapterSettings::default()
    }
}

/// Opens the adapter.
async fn open(settings: AdapterSettings) -> kampus_common::Result<Arc<dyn Mailer>> {
    kampus_mail::open(&[kampus_mail_smtp::ADAPTER], settings).await
}

#[tokio::test]
async fn a_server_that_speaks_the_protocol_conforms() {
    let (address, kept) = fake_server().await;
    let mailer = open(settings(format!("smtp://{address}"))).await.unwrap();
    conformance::run(mailer.as_ref(), &kept).await;
    let from = kept.received().await[0]
        .data
        .lines()
        .find(|l| l.starts_with("From:"))
        .unwrap()
        .to_owned();
    assert!(from.contains("no-reply@kampus.test"), "{from}");
}

#[tokio::test]
async fn a_mailbox_the_server_does_not_know_is_rejected_for_good() {
    let (address, _) = fake_server().await;
    let mailer = open(settings(format!("smtp://{address}"))).await.unwrap();
    let mail = Mail::new(vec![UNKNOWN.to_owned()], "Hi", "Hi").unwrap();
    assert_eq!(mailer.send(&mail).await.unwrap_err().code(), "mail.rejected");
}

#[tokio::test]
async fn an_unreachable_server_is_unavailable_and_settings_are_checked() {
    let mailer = open(settings("smtp://127.0.0.1:1".to_owned())).await.unwrap();
    assert_eq!(mailer.check().await.unwrap_err().code(), "mail.failed");
    let mail = Mail::new(vec!["lea@example.org".to_owned()], "Hi", "Hi").unwrap();
    assert_eq!(mailer.send(&mail).await.unwrap_err().code(), "mail.failed");

    let without_sender = AdapterSettings {
        url: "smtp://127.0.0.1:1".to_owned(),
        ..AdapterSettings::default()
    };
    assert_eq!(
        open(without_sender).await.unwrap_err().code(),
        "mail.settings_invalid"
    );
    let mut half = settings("smtp://127.0.0.1:1".to_owned());
    half.options.insert("username".to_owned(), "kampus".to_owned());
    assert_eq!(open(half).await.unwrap_err().code(), "mail.settings_invalid");
    assert_eq!(
        open(settings("ftp://127.0.0.1:1".to_owned()))
            .await
            .unwrap_err()
            .code(),
        "mail.adapter_unknown"
    );
}

/// What Mailpit received, read from its web interface.
struct Mailpit(String);

#[async_trait]
impl Inbox for Mailpit {
    async fn received(&self) -> Vec<Received> {
        let client = reqwest::Client::new();
        let list: serde_json::Value = client
            .get(format!("{}/api/v1/messages?limit=500", self.0))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let mut found = Vec::new();
        for message in list["messages"].as_array().unwrap().iter().rev() {
            let id = message["ID"].as_str().unwrap();
            let data = client
                .get(format!("{}/api/v1/message/{id}/raw", self.0))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap()
                .replace("\r\n", "\n")
                .replace('\n', "\r\n");
            let to = message["To"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["Address"].as_str().unwrap().to_owned())
                .collect();
            found.push(Received { to, data });
        }
        found
    }
}

#[tokio::test]
async fn a_real_server_conforms_when_one_is_given() {
    if let (Ok(url), Ok(web)) = (
        std::env::var("KAMPUS_TEST_SMTP_URL"),
        std::env::var("KAMPUS_TEST_MAILPIT_URL"),
    ) {
        let mailer = open(settings(url)).await.unwrap();
        conformance::run(mailer.as_ref(), &Mailpit(web)).await;
    }
}
