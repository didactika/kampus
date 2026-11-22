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

//! The OpenAPI document of the API this build serves. It is read from the routes the modules really mounted,
//! so it lists exactly what answers, and never a route that does not exist: the router is the single source.
//! The paths and methods come from the debug description of the router, which a test pins so a change in the
//! format of the framework fails the build instead of the document. What each operation takes and answers
//! comes from the operations each module declares next to its routes, with the JSON schemas of the Rust
//! types of its bodies; [`coverage`] tells a route without an operation and an operation without a route.

use std::collections::{BTreeMap, BTreeSet};

use axum::Router;
use kampus_module::http::Problem;
use kampus_module::{Content, Module, Operation};
use schemars::generate::SchemaSettings;
use schemars::{JsonSchema, SchemaGenerator};
use serde_json::{Map, Value, json};

/// The methods a route can answer, in the order the document lists them.
const METHODS: [&str; 7] = ["get", "post", "put", "patch", "delete", "options", "trace"];

/// Where the schemas of the document live, as a JSON pointer.
const SCHEMAS: &str = "/components/schemas";

/// The methods each path answers, read from the router.
#[must_use]
pub fn routes_of(router: &Router) -> BTreeMap<String, BTreeSet<&'static str>> {
    let text = format!("{router:?}");
    let Some(start) = text.find("path_router") else {
        return BTreeMap::new();
    };
    let text = &text[start..];
    let end = text.find("fallback_router").unwrap_or(text.len());
    let text = &text[..end];
    let paths = ids_and_paths(text);
    let mut found: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    let Some(routes_at) = text.find("routes: {") else {
        return found;
    };
    let entries = &text[routes_at..text.find("node: Node").unwrap_or(text.len())];
    for entry in entries.split("RouteId(").skip(1) {
        let Some((id, rest)) = entry.split_once(')') else {
            continue;
        };
        let (Ok(id), true) = (id.parse::<u32>(), rest.contains("MethodRouter {")) else {
            continue;
        };
        let Some(path) = paths.get(&id) else {
            continue;
        };
        for method in METHODS {
            if !rest.contains(&format!("{method}: None")) && rest.contains(&format!("{method}: ")) {
                found.entry(path.clone()).or_default().insert(method);
            }
        }
    }
    found
}

/// The path of each route id, from the `paths` of the router.
fn ids_and_paths(text: &str) -> BTreeMap<u32, String> {
    let mut paths = BTreeMap::new();
    let Some(at) = text.find("paths: {") else {
        return paths;
    };
    let body = &text[at + "paths: {".len()..];
    let body = &body[..body.find("\"}").map_or(body.len(), |i| i + 1)];
    for part in body.split("RouteId(").skip(1) {
        let Some((id, rest)) = part.split_once("): \"") else {
            continue;
        };
        if let (Ok(id), Some((path, _))) = (id.parse::<u32>(), rest.split_once('"')) {
            paths.insert(id, path.to_owned());
        }
    }
    paths
}

/// The names of the parameters of a path (`{id}`).
fn parameters(path: &str) -> Vec<Value> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')))
        .map(|name| {
            json!({ "name": name.trim_start_matches('*'), "in": "path", "required": true, "schema": { "type": "string" } })
        })
        .collect()
}

/// A name for an operation: the method and the path, so it is unique and stable.
fn operation_id(method: &str, path: &str) -> String {
    let words: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.trim_matches(|c| c == '{' || c == '}')
                .replace(['-', '.', '*'], "_")
        })
        .collect();
    format!("{method}_{}", words.join("_"))
}

/// The module a path of the API belongs to: its first segment.
fn module_of(path: &str) -> &str {
    path.trim_start_matches('/').split('/').next().unwrap_or_default()
}

/// The operation a module declares for a method on a path of the API.
fn declared<'a>(modules: &[&'a Module], method: &str, path: &str) -> Option<&'a Operation> {
    let module = modules.iter().find(|m| m.name == module_of(path))?;
    let inner = path.get(module.name.len() + 1..)?;
    module
        .operations
        .iter()
        .flat_map(|list| list.iter())
        .find(|op| op.method.as_str() == method && op.path == inner)
}

/// How the routes and the operations the modules declare match.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Coverage {
    /// The routes no module declares an operation for, as `METHOD /path`.
    pub undocumented: Vec<String>,
    /// The operations no route answers, as `METHOD /path`.
    pub unrouted: Vec<String>,
    /// The operations whose success answer is not documented, as `METHOD /path`.
    pub without_answer: Vec<String>,
}

/// Compares the routes of the router with the operations the modules declare.
#[must_use]
pub fn coverage(router: &Router, modules: &[&Module]) -> Coverage {
    let mounted = routes_of(router);
    let mut coverage = Coverage::default();
    for (path, methods) in &mounted {
        for method in methods {
            if declared(modules, method, path).is_none() {
                coverage
                    .undocumented
                    .push(format!("{} {path}", method.to_uppercase()));
            }
        }
    }
    for module in modules {
        for op in module.operations.iter().flat_map(|list| list.iter()) {
            let path = format!("/{}{}", module.name, op.path);
            let name = format!("{} {path}", op.method.as_str().to_uppercase());
            if !mounted
                .get(&path)
                .is_some_and(|methods| methods.contains(op.method.as_str()))
            {
                coverage.unrouted.push(name.clone());
            }
            if !op.documents_answer() {
                coverage.without_answer.push(name);
            }
        }
    }
    coverage
}

/// The generators of the schemas: requests as they are read, inlined; answers as they are written, by
/// reference to the components.
struct Generators {
    /// The schemas of requests.
    requests: SchemaGenerator,
    /// The schemas of answers.
    responses: SchemaGenerator,
}

impl Generators {
    /// Generators that place what they refer to under the components of the document.
    fn new() -> Self {
        let settings = |inline: bool| {
            SchemaSettings::draft2020_12().with(|s| {
                s.definitions_path = SCHEMAS.into();
                s.meta_schema = None;
                s.inline_subschemas = inline;
            })
        };
        Self {
            requests: settings(true).for_deserialize().into_generator(),
            responses: settings(false).for_serialize().into_generator(),
        }
    }

    /// The schema of a type in an answer.
    fn response<T: JsonSchema>(&mut self) -> Value {
        self.responses.subschema_for::<T>().to_value()
    }

    /// Every schema referred to, for the components; a request type only recursion left out of line keeps
    /// the name an answer took first.
    fn into_components(mut self) -> Map<String, Value> {
        let mut components = self.responses.take_definitions(true);
        for (name, schema) in self.requests.take_definitions(true) {
            components.entry(name).or_insert(schema);
        }
        components
            .into_iter()
            .map(|(name, schema)| (name, object(schema)))
            .collect()
    }
}

/// A schema as an object: OpenAPI takes no boolean schema, so `true` (anything) becomes `{}`.
fn object(schema: Value) -> Value {
    match schema {
        Value::Bool(true) => json!({}),
        Value::Bool(false) => json!({ "not": {} }),
        other => other,
    }
}

/// The content of a request or an answer, `None` when it carries nothing.
fn content(content: Content, generator: &mut SchemaGenerator) -> Option<Value> {
    match content {
        Content::Empty | Content::Redirect => None,
        Content::Json(schema) => {
            Some(json!({ "application/json": { "schema": object(schema(generator).to_value()) } }))
        }
        Content::Media(media) => {
            Some(json!({ (media): { "schema": { "type": "string", "contentMediaType": media } } }))
        }
    }
}

/// The description of a success answer.
fn answer_description(op: &Operation) -> &'static str {
    match (op.response, op.status) {
        (Content::Redirect, _) => "A redirect to the address in `Location`",
        (_, 201) => "Created",
        (_, 202) => "Accepted",
        (_, 204) => "Done, with nothing to answer",
        _ => "Success",
    }
}

/// The success answers of an operation; `2XX` alone when no module declares it.
fn answers(op: Option<&Operation>, generators: &mut Generators) -> Value {
    let Some(op) = op else {
        return json!({ "2XX": { "description": "Success" } });
    };
    let mut answer = json!({ "description": answer_description(op) });
    if let Some(content) = content(op.response, &mut generators.responses) {
        answer["content"] = content;
    }
    if matches!(op.response, Content::Redirect) {
        answer["headers"] = json!({ "Location": { "schema": { "type": "string" } } });
    }
    json!({ (op.status.to_string()): answer })
}

/// The operation object of a method on a path.
fn operation(
    method: &str,
    path: &str,
    op: Option<&Operation>,
    generators: &mut Generators,
    problem: &Value,
) -> Value {
    let mut parameters = parameters(path);
    if method != "get" {
        parameters.push(json!({
            "name": "x-kampus", "in": "header", "required": true, "schema": { "type": "string" },
            "description": "Any value; a request that changes something with a session cookie must carry it."
        }));
    }
    let mut responses = answers(op, generators);
    responses["default"] = json!({
        "description": "A problem (RFC 9457)",
        "content": { "application/problem+json": { "schema": problem } }
    });
    let summary = op.map_or_else(
        || format!("{} {path}", method.to_uppercase()),
        |op| op.summary.to_owned(),
    );
    let mut operation = json!({
        "operationId": operation_id(method, path),
        "summary": summary,
        "tags": [module_of(path)],
        "parameters": parameters,
        "security": [{ "session": [] }, { "token": [] }],
        "responses": responses,
    });
    if let Some(body) = op.and_then(|op| content(op.request, &mut generators.requests)) {
        operation["requestBody"] = json!({ "required": true, "content": body });
    }
    operation
}

/// The document of the routes mounted under `prefix`, with the module each one belongs to and what each
/// operation takes and answers.
#[must_use]
pub fn document(router: &Router, prefix: &str, modules: &[&Module], version: &str) -> Value {
    let mut generators = Generators::new();
    let problem = generators.response::<Problem>();
    let mut paths = Map::new();
    let mut used = BTreeSet::new();
    for (path, methods) in routes_of(router) {
        let mut item = Map::new();
        for method in METHODS.iter().filter(|m| methods.contains(*m)) {
            used.insert(module_of(&path).to_owned());
            let op = declared(modules, method, &path);
            item.insert(
                (*method).to_owned(),
                operation(method, &path, op, &mut generators, &problem),
            );
        }
        paths.insert(path, Value::Object(item));
    }
    let tags: Vec<Value> = modules
        .iter()
        .filter(|m| used.contains(m.name))
        .map(|m| json!({ "name": m.name, "description": m.summary }))
        .collect();
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Kampus API",
            "version": version,
            "description": "The routes this installation serves, read from the modules it was built with."
        },
        "servers": [{ "url": prefix }],
        "tags": tags,
        "paths": paths,
        "components": {
            "securitySchemes": {
                "session": { "type": "apiKey", "in": "cookie", "name": "kampus_session" },
                "token": { "type": "http", "scheme": "bearer" }
            },
            "schemas": generators.into_components()
        }
    })
}
