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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The OpenAPI document is read from the routes the router really has.

use axum::Router;
use axum::routing::{get, post};
use kampus_server::http::openapi::{document, routes_of};

/// A handler that does nothing.
async fn nothing() {}

/// Two modules with routes of several methods, one of them nested.
fn router() -> Router {
    let notes = Router::new()
        .route("/notes", get(nothing).post(nothing))
        .route("/notes/{id}", get(nothing));
    let tools = Router::new().route("/run", post(nothing));
    Router::new().nest("/notes", notes).nest("/tools", tools)
}

#[test]
fn the_methods_and_paths_come_from_the_router() {
    let routes = routes_of(&router());
    assert_eq!(
        routes["/notes/notes"].iter().copied().collect::<Vec<_>>(),
        ["get", "post"]
    );
    assert_eq!(
        routes["/notes/notes/{id}"].iter().copied().collect::<Vec<_>>(),
        ["get"]
    );
    assert_eq!(routes["/tools/run"].iter().copied().collect::<Vec<_>>(), ["post"]);
    assert_eq!(routes.len(), 3, "nothing else is listed");
}

#[test]
fn the_document_lists_operations_with_their_parameters_and_the_header_for_changes() {
    let doc = document(&router(), "/api/v1", &[], "0.1.0");
    assert_eq!(doc["openapi"], "3.1.0");
    let read = &doc["paths"]["/notes/notes/{id}"]["get"];
    assert_eq!(read["operationId"], "get_notes_notes_id");
    assert_eq!(read["parameters"][0]["name"], "id");
    assert_eq!(read["parameters"].as_array().unwrap().len(), 1);
    let change = &doc["paths"]["/tools/run"]["post"];
    assert_eq!(change["parameters"][0]["name"], "x-kampus");
    assert_eq!(change["tags"][0], "tools");
}
