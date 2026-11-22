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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Rich text keeps only its elements and safe links, and its plain text reads as the person wrote it.

use kampus_common::text::{plain_of_rich, preview, sanitise_rich};

#[test]
fn rich_text_keeps_its_elements_and_drops_what_runs() {
    let clean = sanitise_rich(
        r#"<p onclick="x()">Hi <strong>all</strong><script>alert(1)</script></p><img src=x onerror=alert(1)><a href="javascript:alert(1)">bad</a> <a href="https://example.org" target="_blank">good</a><!-- note --><iframe src="https://example.org"></iframe>"#,
    );
    assert_eq!(
        clean,
        r#"<p>Hi <strong>all</strong></p><a rel="noopener noreferrer nofollow">bad</a> <a href="https://example.org" rel="noopener noreferrer nofollow">good</a>"#
    );
    assert_eq!(
        sanitise_rich("<ul><li>one</li></ul><pre><code>x &lt; y</code></pre>"),
        "<ul><li>one</li></ul><pre><code>x &lt; y</code></pre>"
    );
    assert_eq!(
        sanitise_rich("a < b & c"),
        "a &lt; b &amp; c",
        "plain text is escaped"
    );
}

#[test]
fn the_plain_text_of_rich_text_reads_as_written() {
    assert_eq!(
        plain_of_rich("<p>Tom &amp; Jerry</p><p>say&nbsp;&lt;hi&gt;</p>"),
        "Tom & Jerry say <hi>"
    );
    assert_eq!(preview("short", 10), "short");
    assert_eq!(preview("a long sentence here", 6), "a long…");
    assert_eq!(preview("áéíóú", 3), "áéí…");
}
