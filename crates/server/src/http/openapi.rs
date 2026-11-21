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
//! format of the framework fails the build instead of the document.

use std::collections::{BTreeMap, BTreeSet};

use axum::Router;
use kampus_module::Module;
use serde_json::{Value, json};

/// The methods a route can answer, in the order the document lists them.
const METHODS: [&str; 7] = ["get", "post", "put", "patch", "delete", "options", "trace"];

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

/// The document of the routes mounted under `prefix`, with the module each one belongs to.
#[must_use]
pub fn document(router: &Router, prefix: &str, modules: &[&Module], version: &str) -> Value {
    let mut paths = serde_json::Map::new();
    let mut used = BTreeSet::new();
    for (path, methods) in routes_of(router) {
        let module = path.trim_start_matches('/').split('/').next().unwrap_or_default();
        let mut item = serde_json::Map::new();
        let parameters = parameters(&path);
        for method in METHODS.iter().filter(|m| methods.contains(*m)) {
            used.insert(module.to_owned());
            let mut operation_parameters = parameters.clone();
            if *method != "get" {
                operation_parameters.push(json!({
                    "name": "x-kampus", "in": "header", "required": true, "schema": { "type": "string" },
                    "description": "Any value; a request that changes something with a session cookie must carry it."
                }));
            }
            item.insert(
                (*method).to_owned(),
                json!({
                    "operationId": operation_id(method, &path),
                    "summary": format!("{} {path}", method.to_uppercase()),
                    "tags": [module],
                    "parameters": operation_parameters,
                    "security": [{ "session": [] }, { "token": [] }],
                    "responses": {
                        "2XX": { "description": "Success" },
                        "default": {
                            "description": "A problem (RFC 9457)",
                            "content": { "application/problem+json": { "schema": { "$ref": "#/components/schemas/Problem" } } }
                        }
                    }
                }),
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
            "schemas": {
                "Problem": {
                    "type": "object",
                    "required": ["code", "status", "title", "request_id"],
                    "properties": {
                        "type": { "type": "string" },
                        "title": { "type": "string" },
                        "status": { "type": "integer" },
                        "code": { "type": "string" },
                        "request_id": { "type": "string" },
                        "fields": { "type": "array", "items": { "type": "object" } },
                        "params": { "type": "object" }
                    }
                }
            }
        }
    })
}
