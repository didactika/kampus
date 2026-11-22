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

//! The one rich-text format of user content: a small subset of HTML, sanitised on write and again on render,
//! and its plain text for previews, email and push.

use std::collections::HashSet;
use std::sync::LazyLock;

/// The elements rich text keeps; everything else is removed and its text kept.
const TAGS: &[&str] = &[
    "p",
    "br",
    "strong",
    "em",
    "u",
    "s",
    "a",
    "ul",
    "ol",
    "li",
    "blockquote",
    "code",
    "pre",
];

/// The schemes a link may use.
const SCHEMES: &[&str] = &["https", "http", "mailto"];

/// The cleaner of rich text, built once.
static CLEANER: LazyLock<ammonia::Builder<'static>> = LazyLock::new(|| {
    let mut builder = ammonia::Builder::empty();
    builder
        .tags(TAGS.iter().copied().collect::<HashSet<_>>())
        .add_tag_attributes("a", ["href"])
        .url_schemes(SCHEMES.iter().copied().collect::<HashSet<_>>())
        .link_rel(Some("noopener noreferrer nofollow"))
        .strip_comments(true);
    builder
});

/// Rich text as it may be stored and shown: only the elements of the format, links to the web or to email
/// only, nothing that runs.
#[must_use]
pub fn sanitise_rich(html: &str) -> String {
    CLEANER.clean(html).to_string()
}

/// The plain text of rich text: elements removed, entities decoded, blank runs folded to one space.
#[must_use]
pub fn plain_of_rich(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut inside = false;
    for c in html.chars() {
        match c {
            '<' => {
                inside = true;
                text.push(' ');
            }
            '>' if inside => inside = false,
            _ if !inside => text.push(c),
            _ => {}
        }
    }
    let decoded = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The first characters of a text, cut at a character boundary, with an ellipsis when it was cut.
#[must_use]
pub fn preview(text: &str, characters: usize) -> String {
    let mut chars = text.chars();
    let cut: String = chars.by_ref().take(characters).collect();
    if chars.next().is_some() {
        format!("{}…", cut.trim_end())
    } else {
        cut
    }
}
