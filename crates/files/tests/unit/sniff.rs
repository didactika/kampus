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

//! The media type of content: read from its first bytes, and what an area accepts.

use kampus_files::support::sniff::{active, allowed, media_type};

#[test]
fn the_content_decides_the_type_not_the_sender() {
    assert_eq!(
        media_type(b"\x89PNG\r\n\x1a\n....", Some("text/plain")),
        "image/png"
    );
    assert_eq!(media_type(b"\xff\xd8\xff\xe0....", None), "image/jpeg");
    assert_eq!(media_type(b"%PDF-1.7 ...", Some("image/png")), "application/pdf");
    assert_eq!(media_type(b"RIFF\0\0\0\0WEBPVP8 ", None), "image/webp");
    assert_eq!(
        media_type(
            b"  <svg xmlns=\"http://www.w3.org/2000/svg\"/>",
            Some("image/png")
        ),
        "image/svg+xml"
    );
    assert_eq!(
        media_type(b"<!DOCTYPE html><p>hi", Some("text/plain")),
        "text/html"
    );
    assert_eq!(media_type(b"a,b\n1,2\n", Some("text/csv")), "text/csv");
    assert_eq!(media_type(b"a,b\n1,2\n", Some("image/png")), "text/plain");
    assert_eq!(
        media_type(&[0, 1, 2, 255, 254], Some("image/png")),
        "application/octet-stream"
    );
}

#[test]
fn an_area_accepts_its_types_and_families_and_active_content_is_known() {
    let images = vec!["image/*".to_owned(), "application/pdf".to_owned()];
    assert!(allowed(&images, "image/png"));
    assert!(allowed(&images, "application/pdf"));
    assert!(!allowed(&images, "text/html"));
    assert!(!allowed(&images, "imagery/png"));
    assert!(active("image/svg+xml") && active("text/html"));
    assert!(!active("image/png"));
}
