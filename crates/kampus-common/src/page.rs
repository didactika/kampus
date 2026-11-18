// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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

//! Cursor pagination (DEC-025): lists are read page by page with an opaque cursor, 50 items by default
//! and 200 at most.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Serialize, Serializer};

use crate::{Error, FieldError, Result};

/// The number of items of a page when the caller does not ask for one.
pub const DEFAULT_LIMIT: u32 = 50;

/// The largest page a caller may ask for.
pub const MAX_LIMIT: u32 = 200;

/// An opaque position in a list, produced by the module that reads the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor(Vec<u8>);

impl Cursor {
    /// Wraps the position a module encoded (for example the sort key and the id of the last item).
    #[must_use]
    pub const fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// The position, for the module that produced it.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The text form that travels in `after` and `next`.
    #[must_use]
    pub fn encode(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.0)
    }

    /// Reads the text form back.
    ///
    /// # Errors
    ///
    /// A `validation` error on the field `after` when the text is not a cursor.
    pub fn decode(text: &str) -> Result<Self> {
        URL_SAFE_NO_PAD
            .decode(text)
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .map(Self)
            .ok_or_else(|| Error::validation("common.page_invalid", vec![FieldError::new("after", "cursor")]))
    }
}

impl Serialize for Cursor {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.encode())
    }
}

/// A request for one page of a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRequest {
    /// How many items the page holds.
    limit: u32,
    /// Where the page starts, after the last item of the previous page.
    after: Option<Cursor>,
}

impl PageRequest {
    /// Builds a page request from the query parameters `limit` and `after`.
    ///
    /// # Errors
    ///
    /// A `validation` error listing every invalid parameter.
    pub fn new(limit: Option<u32>, after: Option<&str>) -> Result<Self> {
        let mut fields = Vec::new();
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        if !(1..=MAX_LIMIT).contains(&limit) {
            fields.push(FieldError::new("limit", "range"));
        }
        let after = match after.map(Cursor::decode).transpose() {
            Ok(after) => after,
            Err(error) => {
                fields.extend_from_slice(error.fields());
                None
            }
        };
        if fields.is_empty() {
            Ok(Self { limit, after })
        } else {
            Err(Error::validation("common.page_invalid", fields).with_param("max", MAX_LIMIT.to_string()))
        }
    }

    /// How many items the page holds.
    #[must_use]
    pub const fn limit(&self) -> u32 {
        self.limit
    }

    /// Where the page starts, if it is not the first one.
    #[must_use]
    pub const fn after(&self) -> Option<&Cursor> {
        self.after.as_ref()
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            after: None,
        }
    }
}

/// One page of a list, with the cursor of the next page when there is one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Page<T> {
    /// The items of the page.
    pub items: Vec<T>,
    /// The cursor of the next page, absent on the last page.
    pub next: Option<Cursor>,
}

#[cfg(test)]
mod tests {
    use super::{Cursor, DEFAULT_LIMIT, Page, PageRequest};

    #[test]
    fn the_default_page_holds_fifty_items() {
        let page = PageRequest::new(None, None).unwrap();
        assert_eq!(page.limit(), DEFAULT_LIMIT);
        assert!(page.after().is_none());
    }

    #[test]
    fn cursors_round_trip() {
        let cursor = Cursor::from_bytes(b"course:0192".to_vec());
        let page = PageRequest::new(Some(10), Some(&cursor.encode())).unwrap();
        assert_eq!(page.after(), Some(&cursor));
    }

    #[test]
    fn every_invalid_parameter_is_reported_at_once() {
        let error = PageRequest::new(Some(500), Some("not a cursor!")).unwrap_err();
        let fields: Vec<_> = error
            .fields()
            .iter()
            .map(|f| (f.field.as_str(), f.rule.as_str()))
            .collect();
        assert_eq!(fields, [("limit", "range"), ("after", "cursor")]);
        assert_eq!(
            PageRequest::new(Some(0), None).unwrap_err().fields()[0].rule,
            "range"
        );
    }

    #[test]
    fn pages_serialise_the_next_cursor_as_text() {
        let page = Page {
            items: vec![1, 2],
            next: Some(Cursor::from_bytes(vec![1, 2, 3])),
        };
        assert_eq!(
            serde_json::to_string(&page).unwrap(),
            r#"{"items":[1,2],"next":"AQID"}"#
        );
        let last: Page<u8> = Page {
            items: vec![],
            next: None,
        };
        assert_eq!(
            serde_json::to_string(&last).unwrap(),
            r#"{"items":[],"next":null}"#
        );
    }
}
