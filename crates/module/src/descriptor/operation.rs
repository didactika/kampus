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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a module says about each operation its routes answer: the method and the path, a summary, and the
//! content of the request and of the success answer, with the JSON schema of the Rust type that carries it.
//! The server builds the OpenAPI document from these and from the routes it mounted; a test fails when a
//! route has no operation or an operation no route, so the two cannot drift apart.

use schemars::{JsonSchema, Schema, SchemaGenerator};

/// Builds the JSON schema of a type, registering the types it refers to in the generator.
pub type SchemaOf = fn(&mut SchemaGenerator) -> Schema;

/// The JSON schema of `T`, as a [`SchemaOf`].
#[must_use]
pub fn schema_of<T: JsonSchema>(generator: &mut SchemaGenerator) -> Schema {
    generator.subschema_for::<T>()
}

/// The HTTP method of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Method {
    /// `GET`.
    Get,
    /// `POST`.
    Post,
    /// `PUT`.
    Put,
    /// `PATCH`.
    Patch,
    /// `DELETE`.
    Delete,
}

impl Method {
    /// The name of the method in an OpenAPI document (`get`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Patch => "patch",
            Self::Delete => "delete",
        }
    }
}

/// What a request or an answer carries.
#[derive(Debug, Clone, Copy)]
pub enum Content {
    /// Nothing.
    Empty,
    /// JSON of the type the schema describes.
    Json(SchemaOf),
    /// Bytes of a media type (`text/css`, `application/octet-stream`).
    Media(&'static str),
    /// A redirect to another address, in `Location`.
    Redirect,
}

/// One operation of a module: a method on a path, written without the prefix of the module as in its routes.
#[derive(Debug, Clone, Copy)]
pub struct Operation {
    /// The method.
    pub method: Method,
    /// The path, with its parameters in braces (`/files/{id}`).
    pub path: &'static str,
    /// What it does, in one line.
    pub summary: &'static str,
    /// What the request carries.
    pub request: Content,
    /// The status of the success answer.
    pub status: u16,
    /// What the success answer carries.
    pub response: Content,
}

impl Operation {
    /// An operation that takes nothing and answers `200` with nothing yet; the builders below say more.
    #[must_use]
    pub const fn new(method: Method, path: &'static str, summary: &'static str) -> Self {
        Self {
            method,
            path,
            summary,
            request: Content::Empty,
            status: 200,
            response: Content::Empty,
        }
    }

    /// A `GET` operation.
    #[must_use]
    pub const fn get(path: &'static str, summary: &'static str) -> Self {
        Self::new(Method::Get, path, summary)
    }

    /// A `POST` operation.
    #[must_use]
    pub const fn post(path: &'static str, summary: &'static str) -> Self {
        Self::new(Method::Post, path, summary)
    }

    /// A `PUT` operation.
    #[must_use]
    pub const fn put(path: &'static str, summary: &'static str) -> Self {
        Self::new(Method::Put, path, summary)
    }

    /// A `PATCH` operation.
    #[must_use]
    pub const fn patch(path: &'static str, summary: &'static str) -> Self {
        Self::new(Method::Patch, path, summary)
    }

    /// A `DELETE` operation.
    #[must_use]
    pub const fn delete(path: &'static str, summary: &'static str) -> Self {
        Self::new(Method::Delete, path, summary)
    }

    /// The request carries JSON of type `T`.
    #[must_use]
    pub const fn body<T: JsonSchema>(mut self) -> Self {
        self.request = Content::Json(schema_of::<T>);
        self
    }

    /// The request carries bytes of a media type.
    #[must_use]
    pub const fn body_media(mut self, media: &'static str) -> Self {
        self.request = Content::Media(media);
        self
    }

    /// The answer is `200` with JSON of type `T`.
    #[must_use]
    pub const fn returns<T: JsonSchema>(self) -> Self {
        self.answers::<T>(200)
    }

    /// The answer is `201` with JSON of type `T`, what was created.
    #[must_use]
    pub const fn creates<T: JsonSchema>(self) -> Self {
        self.answers::<T>(201)
    }

    /// The answer has a status and JSON of type `T`.
    #[must_use]
    pub const fn answers<T: JsonSchema>(mut self, status: u16) -> Self {
        self.status = status;
        self.response = Content::Json(schema_of::<T>);
        self
    }

    /// The answer is `200` with bytes of a media type.
    #[must_use]
    pub const fn returns_media(mut self, media: &'static str) -> Self {
        self.status = 200;
        self.response = Content::Media(media);
        self
    }

    /// The answer is `204`, with nothing.
    #[must_use]
    pub const fn no_content(mut self) -> Self {
        self.status = 204;
        self.response = Content::Empty;
        self
    }

    /// The answer is `202`, with nothing: the work goes on, or its outcome arrives another way.
    #[must_use]
    pub const fn accepted(mut self) -> Self {
        self.status = 202;
        self.response = Content::Empty;
        self
    }

    /// The answer is a redirect with a status (`302`, `303`).
    #[must_use]
    pub const fn redirects(mut self, status: u16) -> Self {
        self.status = status;
        self.response = Content::Redirect;
        self
    }

    /// Whether the success answer is documented: content, nothing on a `202` or `204`, or a redirect.
    #[must_use]
    pub const fn documents_answer(&self) -> bool {
        match self.response {
            Content::Empty => matches!(self.status, 202 | 204),
            Content::Json(_) | Content::Media(_) | Content::Redirect => true,
        }
    }
}
