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

//! The reference of a build: what each of its modules declares (permissions with the role templates that grant
//! them, settings, features, terms, events, notification types, jobs and their schedules), read from the
//! descriptors alone, so the handbook is generated from what the code really has.

use kampus_module::permission::Value as Grant;
use kampus_module::{Module, ValueKind};
use serde_json::{Value, json};

/// What a setting accepts, with its default.
fn accepts(kind: ValueKind) -> Value {
    match kind {
        ValueKind::Boolean { default } => json!({ "type": "boolean", "default": default }),
        ValueKind::Choice { options, default } => {
            json!({ "type": "choice", "options": options, "default": default })
        }
        ValueKind::Integer { min, max, default } => {
            json!({ "type": "integer", "min": min, "max": max, "default": default })
        }
        ValueKind::Text { max, default } => json!({ "type": "text", "max": max, "default": default }),
        ValueKind::RichText { max } => json!({ "type": "rich_text", "max": max, "default": "" }),
    }
}

/// The role templates of every module that allow a permission.
fn granted_by(modules: &[&Module], code: &str) -> Vec<&'static str> {
    let mut roles: Vec<&'static str> = modules
        .iter()
        .flat_map(|m| m.templates.iter())
        .filter(|t| t.values.iter().any(|(p, v)| *p == code && *v == Grant::Allow))
        .map(|t| t.code)
        .collect();
    roles.sort_unstable();
    roles.dedup();
    roles
}

/// The reference of one module.
fn module(modules: &[&Module], m: &Module) -> Value {
    json!({
        "name": m.name,
        "summary": m.summary,
        "permissions": m.permissions.iter().map(|p| json!({
            "code": p.code,
            "kind": p.kind,
            "risks": p.risks,
            "lowest_scope": p.lowest_kind,
            "granted_by": granted_by(modules, p.code),
        })).collect::<Vec<_>>(),
        "settings": m.settings.iter().map(|s| json!({
            "key": s.key,
            "summary": s.summary,
            "accepts": accepts(s.kind),
            "levels": s.levels,
            "permission": s.permission,
            "restart_required": s.restart_required,
        })).collect::<Vec<_>>(),
        "flags": m.flags.iter().map(|f| json!({ "code": f.code, "summary": f.summary, "default": f.default })).collect::<Vec<_>>(),
        "terms": m.terms.iter().map(|t| json!({
            "code": t.code,
            "forms": t.forms.iter().map(|f| json!({ "language": f.language, "singular": f.singular, "plural": f.plural })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "events": m.events.iter().map(|e| json!({
            "code": e.code,
            "version": e.version,
            "summary": e.summary,
            "fields": e.fields,
            "personal_fields": e.personal_fields,
            "audited": e.audited,
        })).collect::<Vec<_>>(),
        "notification_types": m.notification_types.iter().map(|n| json!({
            "code": n.code,
            "summary": n.summary,
            "channels": n.channels,
            "required": n.required,
        })).collect::<Vec<_>>(),
        "jobs": m.jobs.iter().map(|j| json!({
            "code": j.code,
            "summary": j.summary,
            "max_attempts": j.max_attempts,
            "schedule": m.schedules.iter().find(|s| s.job == j.code).map(|s| json!({ "cron": s.cron, "time_zone": s.time_zone })),
        })).collect::<Vec<_>>(),
    })
}

/// The reference of every module of the build, in the order of the build.
#[must_use]
pub fn reference(modules: &[&Module]) -> Value {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "modules": modules.iter().map(|m| module(modules, m)).collect::<Vec<_>>(),
    })
}
