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
// @package   kampus-scan-clamd
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The scanning conformance suite against a clamd that speaks the protocol and finds the test file, and
//! against a real `ClamAV` when `KAMPUS_TEST_CLAMD_URL` names one.

use kampus_common::AdapterSettings;
use kampus_scan::{EICAR, conformance};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Reads up to and without a terminating zero.
async fn command(stream: &mut TcpStream) -> Vec<u8> {
    let mut command = Vec::new();
    let mut byte = [0_u8; 1];
    while stream.read_exact(&mut byte).await.is_ok() && byte[0] != 0 {
        command.push(byte[0]);
    }
    command
}

/// Answers one connection as clamd would.
async fn answer(mut stream: TcpStream) {
    let command = command(&mut stream).await;
    if command == b"zPING" {
        stream.write_all(b"PONG\0").await.unwrap();
        return;
    }
    let mut content = Vec::new();
    loop {
        let mut size = [0_u8; 4];
        stream.read_exact(&mut size).await.unwrap();
        let size = usize::try_from(u32::from_be_bytes(size)).unwrap();
        if size == 0 {
            break;
        }
        let mut chunk = vec![0_u8; size];
        stream.read_exact(&mut chunk).await.unwrap();
        content.extend_from_slice(&chunk);
    }
    let found = content.windows(EICAR.len()).any(|w| w == EICAR);
    let reply: &[u8] = if found {
        b"stream: Eicar-Test-Signature FOUND\0"
    } else {
        b"stream: OK\0"
    };
    stream.write_all(reply).await.unwrap();
}

/// A clamd on a free local port; returns its address.
async fn fake_clamd() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(answer(stream));
        }
    });
    address
}

/// Opens the adapter on a URL.
async fn open(url: String) -> std::sync::Arc<dyn kampus_scan::Scanner> {
    kampus_scan::open(
        &[kampus_scan_clamd::ADAPTER],
        AdapterSettings {
            url,
            ..AdapterSettings::default()
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn a_clamd_that_speaks_the_protocol_conforms() {
    let scanner = open(format!("clamd://{}", fake_clamd().await)).await;
    conformance::run(scanner.as_ref()).await;
}

#[tokio::test]
async fn a_real_clamav_conforms_when_one_is_given() {
    if let Ok(url) = std::env::var("KAMPUS_TEST_CLAMD_URL") {
        conformance::run(open(url).await.as_ref()).await;
    }
}

#[tokio::test]
async fn an_unreachable_scanner_is_unavailable() {
    let scanner = open("clamd://127.0.0.1:1".to_owned()).await;
    let error = scanner.check().await.unwrap_err();
    assert_eq!(error.code(), "scan.failed");
}
