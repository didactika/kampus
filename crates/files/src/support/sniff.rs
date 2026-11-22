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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The media type of content, read from its first bytes rather than taken from what the sender says, and
//! whether an area accepts it.

/// The media type of content: from its signature when it has one, the declared type when that is a kind of
/// text the content could be, plain text for other text, and `application/octet-stream` otherwise.
#[must_use]
pub fn media_type(content: &[u8], declared: Option<&str>) -> String {
    if let Some(found) = signature(content) {
        return found.to_owned();
    }
    let text = std::str::from_utf8(content).is_ok() && !content.contains(&0);
    if !text {
        return "application/octet-stream".to_owned();
    }
    let start = String::from_utf8_lossy(&content[..content.len().min(512)]).to_ascii_lowercase();
    let start = start.trim_start();
    if start.starts_with("<svg") || (start.starts_with("<?xml") && start.contains("<svg")) {
        return "image/svg+xml".to_owned();
    }
    if start.starts_with("<!doctype html") || start.starts_with("<html") {
        return "text/html".to_owned();
    }
    match declared {
        Some(declared @ ("text/csv" | "text/markdown" | "application/json")) => declared.to_owned(),
        _ => "text/plain".to_owned(),
    }
}

/// The media type a signature at the start of content gives.
fn signature(content: &[u8]) -> Option<&'static str> {
    const SIGNATURES: &[(&[u8], &str)] = &[
        (b"\x89PNG\r\n\x1a\n", "image/png"),
        (b"\xff\xd8\xff", "image/jpeg"),
        (b"GIF87a", "image/gif"),
        (b"GIF89a", "image/gif"),
        (b"%PDF-", "application/pdf"),
        (b"PK\x03\x04", "application/zip"),
        (b"ID3", "audio/mpeg"),
        (b"OggS", "audio/ogg"),
    ];
    if content.len() >= 12 && &content[..4] == b"RIFF" && &content[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if content.len() >= 12 && &content[4..8] == b"ftyp" {
        return Some("video/mp4");
    }
    SIGNATURES
        .iter()
        .find(|(magic, _)| content.starts_with(magic))
        .map(|(_, media)| *media)
}

/// Whether an area that accepts these types, exact or as a family such as `image/*`, accepts a type.
#[must_use]
pub fn allowed(allowed: &[String], media_type: &str) -> bool {
    allowed.iter().any(|a| {
        a == media_type
            || a == "*/*"
            || a.strip_suffix("/*")
                .is_some_and(|family| media_type.split('/').next() == Some(family))
    })
}

/// Whether content of a type could run in a browser, so it is only ever served as an attachment.
#[must_use]
pub fn active(media_type: &str) -> bool {
    matches!(
        media_type,
        "text/html"
            | "image/svg+xml"
            | "application/xhtml+xml"
            | "text/javascript"
            | "application/javascript"
    )
}

/// Whether content of a type can never run in a browser, even without the headers the API serves it with,
/// so the store may serve it directly: raster images, PDF, audio, video and plain text.
#[must_use]
pub fn inert(media_type: &str) -> bool {
    matches!(
        media_type,
        "image/png"
            | "image/jpeg"
            | "image/gif"
            | "image/webp"
            | "application/pdf"
            | "audio/mpeg"
            | "audio/ogg"
            | "video/mp4"
            | "text/plain"
    )
}
