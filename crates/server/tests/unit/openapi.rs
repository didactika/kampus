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

//! The OpenAPI document is read from the routes the router really has, and from the operations the modules
//! declare next to them, which must match the routes one to one.

use axum::Router;
use axum::routing::{get, post};
use kampus_module::{Module, Operation, ServiceContext};
use kampus_server::http::openapi::{coverage, document, routes_of};
use kampus_server::wiring::registry::MODULES;
use serde_json::Value;

/// A handler that does nothing.
async fn nothing() {}

/// A note to write.
#[derive(serde::Deserialize, schemars::JsonSchema)]
#[allow(dead_code, reason = "only its schema is read")]
struct NewNote {
    /// What it says.
    text: String,
}

/// A note as it is answered.
#[derive(serde::Serialize, schemars::JsonSchema)]
struct Note {
    /// What it says.
    text: String,
    /// Its author, when known.
    author: Option<String>,
}

/// The operations of the notes module of the tests; one route has none.
const NOTES: &[Operation] = &[
    Operation::get("/notes", "The notes").returns::<Vec<Note>>(),
    Operation::post("/notes", "Write a note")
        .body::<NewNote>()
        .creates::<Note>(),
    Operation::delete("/gone", "Not routed").no_content(),
];

/// The operations of the tools module of the tests.
const TOOLS: &[Operation] = &[Operation::post("/run", "Run").body_media("text/plain")];

/// A module of the tests named `name` with these operations.
const fn module(name: &'static str, operations: &'static [&'static [Operation]]) -> Module {
    Module {
        name,
        routes: None,
        operations,
        ..kampus_files::MODULE
    }
}

/// The notes module of the tests.
const NOTES_MODULE: Module = module("notes", &[NOTES]);

/// The tools module of the tests.
const TOOLS_MODULE: Module = module("tools", &[TOOLS]);

/// Two modules with routes of several methods, one of them nested.
fn router() -> Router {
    let notes = Router::new()
        .route("/notes", get(nothing).post(nothing))
        .route("/notes/{id}", get(nothing));
    let tools = Router::new().route("/run", post(nothing));
    Router::new().nest("/notes", notes).nest("/tools", tools)
}

/// Every module of the build, with the routes it mounts on services it does not have.
fn every_route() -> Router {
    let context = ServiceContext::default();
    MODULES
        .iter()
        .fold(Router::new(), |router, module| match module.routes {
            Some(routes) => router.nest(&format!("/{}", module.name), routes(&context)),
            None => router,
        })
}

/// Whether every `schema` of a media type, and every schema of the components, is an object, as OpenAPI
/// requires; JSON Schema alone also allows `true` and `false`.
fn schemas_are_objects(doc: &Value) -> bool {
    fn walk(value: &Value) -> bool {
        match value {
            Value::Object(map) => map
                .iter()
                .all(|(key, value)| (key != "schema" || value.is_object()) && walk(value)),
            Value::Array(items) => items.iter().all(walk),
            _ => true,
        }
    }
    walk(&doc["paths"])
        && doc["components"]["schemas"]
            .as_object()
            .unwrap()
            .values()
            .all(Value::is_object)
}

/// Every `$ref` under a value.
fn references(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                match (key.as_str(), value) {
                    ("$ref", Value::String(reference)) => found.push(reference.clone()),
                    _ => references(value, found),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| references(item, found)),
        _ => {}
    }
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

#[test]
fn the_bodies_come_from_the_operations_the_modules_declare() {
    let doc = document(&router(), "/api/v1", &[&NOTES_MODULE, &TOOLS_MODULE], "0.1.0");
    let write = &doc["paths"]["/notes/notes"]["post"];
    assert_eq!(write["summary"], "Write a note");
    let request = &write["requestBody"]["content"]["application/json"]["schema"];
    assert_eq!(request["type"], "object", "a request is inlined");
    assert_eq!(request["required"][0], "text");
    let created = &write["responses"]["201"]["content"]["application/json"]["schema"];
    assert_eq!(created["$ref"], "#/components/schemas/Note");
    let note = &doc["components"]["schemas"]["Note"];
    assert_eq!(note["properties"]["text"]["description"], "What it says.");
    let list =
        &doc["paths"]["/notes/notes"]["get"]["responses"]["200"]["content"]["application/json"]["schema"];
    assert_eq!(list["type"], "array");
    let problem = &write["responses"]["default"]["content"]["application/problem+json"]["schema"];
    assert_eq!(problem["$ref"], "#/components/schemas/Problem");
    let problem = &doc["components"]["schemas"]["Problem"];
    for field in [
        "type",
        "title",
        "status",
        "code",
        "params",
        "fields",
        "request_id",
    ] {
        assert!(problem["properties"][field].is_object(), "{field}");
    }
    let run = &doc["paths"]["/tools/run"]["post"];
    assert!(run["requestBody"]["content"]["text/plain"].is_object());
    assert!(
        doc["paths"]["/notes/notes/{id}"]["get"]["responses"]["2XX"].is_object(),
        "a route without an operation keeps a bare answer"
    );
}

#[test]
fn coverage_tells_routes_without_operations_and_operations_without_routes() {
    let found = coverage(&router(), &[&NOTES_MODULE, &TOOLS_MODULE]);
    assert_eq!(found.undocumented, ["GET /notes/notes/{id}"]);
    assert_eq!(found.unrouted, ["DELETE /notes/gone"]);
    assert_eq!(found.without_answer, ["POST /tools/run"]);
}

#[test]
fn every_route_of_the_build_has_one_operation_with_its_answer() {
    let found = coverage(&every_route(), MODULES);
    assert!(
        found.undocumented.is_empty(),
        "routes without an operation:\n{}",
        found.undocumented.join("\n")
    );
    assert!(
        found.unrouted.is_empty(),
        "operations without a route:\n{}",
        found.unrouted.join("\n")
    );
    assert!(
        found.without_answer.is_empty(),
        "operations without a documented answer:\n{}",
        found.without_answer.join("\n")
    );
}

#[test]
fn the_document_of_the_build_is_well_formed_and_every_reference_resolves() {
    let doc = document(&every_route(), "/api/v1", MODULES, "0.1.0");
    let paths = doc["paths"].as_object().unwrap();
    assert!(!paths.is_empty());
    let mut ids = std::collections::BTreeSet::new();
    let mut bodies = 0;
    for (path, item) in paths {
        for (method, operation) in item.as_object().unwrap() {
            let name = format!("{method} {path}");
            assert!(
                ids.insert(operation["operationId"].as_str().unwrap().to_owned()),
                "{name}"
            );
            assert!(!operation["summary"].as_str().unwrap().is_empty(), "{name}");
            let responses = operation["responses"].as_object().unwrap();
            assert!(responses.contains_key("default"), "{name}");
            assert!(
                responses
                    .keys()
                    .any(|code| code.len() == 3 && code.parse::<u16>().is_ok()),
                "{name} has no success status"
            );
            if operation["requestBody"].is_object() {
                bodies += 1;
                assert!(
                    !operation["requestBody"]["content"]
                        .as_object()
                        .unwrap()
                        .is_empty(),
                    "{name}"
                );
            }
        }
    }
    assert!(bodies > 0, "some operations take a body");
    assert!(schemas_are_objects(&doc), "a schema is a boolean");
    let schemas = doc["components"]["schemas"].as_object().unwrap();
    let mut found = Vec::new();
    references(&doc, &mut found);
    for reference in found {
        let name = reference
            .strip_prefix("#/components/schemas/")
            .unwrap_or_else(|| panic!("{reference} points outside the schemas"));
        assert!(schemas.contains_key(name), "{reference} does not resolve");
    }
}
