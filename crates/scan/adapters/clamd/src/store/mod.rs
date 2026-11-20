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

//! A client of clamd, `ClamAV`'s daemon, over TCP: content goes in chunks with `INSTREAM`, and the answer
//! names what was found; `PING` checks it answers.

use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use kampus_common::{Error, Result};
use kampus_scan::{Scanner, Verdict};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// The largest chunk sent at once, below clamd's default `StreamMaxLength` per chunk.
const CHUNK: usize = 64 * 1024;

/// How long a scan may take.
const TIMEOUT: Duration = Duration::from_secs(120);

/// The longest answer read.
const MAX_ANSWER: usize = 4096;

/// A clamd reached at an address.
#[derive(Debug, Clone)]
pub struct Clamd {
    /// `host:port`.
    address: String,
}

/// The error of a scanner that cannot be reached or answers what it should not.
fn failed(what: &str) -> Error {
    Error::unavailable("scan.failed").with_param("cause", what.to_owned())
}

impl Clamd {
    /// A client of the clamd at `host:port`.
    #[must_use]
    pub const fn new(address: String) -> Self {
        Self { address }
    }

    /// Sends a command with its content and reads the answer, up to its terminating zero.
    async fn exchange(&self, command: &[u8], content: Option<&[u8]>) -> Result<String> {
        let talk = async {
            let mut stream = TcpStream::connect(&self.address)
                .await
                .map_err(|_| failed("connect"))?;
            stream.write_all(command).await.map_err(|_| failed("write"))?;
            if let Some(content) = content {
                for chunk in content.chunks(CHUNK) {
                    let size = u32::try_from(chunk.len()).map_err(|_| failed("chunk"))?;
                    stream
                        .write_all(&size.to_be_bytes())
                        .await
                        .map_err(|_| failed("write"))?;
                    stream.write_all(chunk).await.map_err(|_| failed("write"))?;
                }
                stream
                    .write_all(&[0, 0, 0, 0])
                    .await
                    .map_err(|_| failed("write"))?;
            }
            let mut answer = Vec::new();
            let mut buffer = [0_u8; 512];
            loop {
                let read = stream.read(&mut buffer).await.map_err(|_| failed("read"))?;
                if read == 0 {
                    break;
                }
                answer.extend_from_slice(&buffer[..read]);
                if answer.contains(&0) || answer.len() > MAX_ANSWER {
                    break;
                }
            }
            let end = answer.iter().position(|b| *b == 0).unwrap_or(answer.len());
            Ok(String::from_utf8_lossy(&answer[..end]).trim().to_owned())
        };
        tokio::time::timeout(TIMEOUT, talk)
            .await
            .map_err(|_| failed("timeout"))?
    }
}

/// What an answer of `INSTREAM` says: `stream: OK`, `stream: <name> FOUND`, or an error.
fn verdict(answer: &str) -> Result<Verdict> {
    let body = answer.strip_prefix("stream:").map_or(answer, str::trim);
    if body == "OK" {
        return Ok(Verdict::Clean);
    }
    match body.strip_suffix(" FOUND") {
        Some(name) => Ok(Verdict::Infected(name.trim().to_owned())),
        None => Err(failed(body)),
    }
}

#[async_trait]
impl Scanner for Clamd {
    async fn scan(&self, content: Bytes) -> Result<Verdict> {
        let answer = self.exchange(b"zINSTREAM\0", Some(&content)).await?;
        verdict(&answer)
    }

    async fn check(&self) -> Result<()> {
        if self.exchange(b"zPING\0", None).await? == "PONG" {
            Ok(())
        } else {
            Err(failed("ping"))
        }
    }
}
