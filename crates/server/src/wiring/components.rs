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

//! The components of this build, from the registries: core modules, plugins and adapters, what each
//! says about itself, and the rules they must follow together.

use kampus_common::PLATFORM;
use kampus_common::component::{self, Manifest, Problem, Registered};
use kampus_db::ComponentRecord;

use super::{adapters, registry};

/// The name the platform is registered with.
pub const PLATFORM_COMPONENT: &str = "kampus";

/// A component of this build.
#[derive(Debug, Clone, Copy)]
pub struct BuildComponent {
    /// What it says about itself.
    pub manifest: &'static Manifest,
    /// `module`, `plugin` or `adapter`.
    pub kind: &'static str,
    /// Its plugin type, or the port of an adapter.
    pub plugin_type: Option<&'static str>,
    /// Whether it comes from outside the repository.
    pub external: bool,
    /// The schema its steps create, when it is a module.
    pub schema: Option<&'static str>,
}

impl BuildComponent {
    /// Its name.
    #[must_use]
    pub fn name(&self) -> String {
        self.manifest.component()
    }
}

/// The plugin types of this build: the ports, whose plugins are adapters, the types with a crate of their own, and
/// those modules define.
#[must_use]
pub fn plugin_types() -> Vec<&'static str> {
    let mut types = vec!["cache", "storage", kampus_widget::TYPE];
    types.extend(
        registry::MODULES
            .iter()
            .flat_map(|m| m.plugin_types.iter().copied()),
    );
    types
}

/// Every component of this build: modules in dependency order, then adapters.
#[must_use]
pub fn build() -> Vec<BuildComponent> {
    let types = plugin_types();
    let mut found = Vec::new();
    for module in registry::MODULES {
        let name = module.manifest.component();
        let plugin_type = types.iter().copied().find(|t| name.starts_with(&format!("{t}_")));
        found.push(BuildComponent {
            manifest: &module.manifest,
            kind: if plugin_type.is_some() { "plugin" } else { "module" },
            plugin_type,
            external: registry::EXTERNAL.contains(&module.manifest.package),
            schema: Some(module.name),
        });
    }
    let ports = [
        (
            "cache",
            adapters::CACHE.iter().map(|a| &a.manifest).collect::<Vec<_>>(),
        ),
        ("storage", adapters::STORAGE.iter().map(|a| &a.manifest).collect()),
    ];
    for (port, manifests) in ports {
        for manifest in manifests {
            found.push(BuildComponent {
                manifest,
                kind: "adapter",
                plugin_type: Some(port),
                external: adapters::EXTERNAL.contains(&manifest.package),
                schema: None,
            });
        }
    }
    found
}

/// What is wrong with the components of this build, for the release it runs: blocking problems stop
/// the platform, the rest are shown.
#[must_use]
pub fn problems(platform: &str) -> Vec<Problem> {
    let components = build();
    let registered: Vec<Registered<'_>> = components
        .iter()
        .map(|c| Registered {
            manifest: c.manifest,
            plugin_type: c.plugin_type,
            external: c.external,
        })
        .collect();
    let mut found = component::check(&registered, &plugin_types(), platform);
    for module in registry::MODULES {
        if module.name != module.manifest.component() {
            found.push(Problem {
                component: module.manifest.component(),
                message: format!(
                    "its descriptor names it {}: the name comes from the package, {}",
                    module.name, module.manifest.package
                ),
                blocking: true,
            });
        }
    }
    found
}

/// The components as the upgrade registers them: the platform first, which owns the steps of the
/// database layer, then every component of the build.
#[must_use]
pub fn records() -> Vec<ComponentRecord> {
    let platform = ComponentRecord {
        name: PLATFORM_COMPONENT.to_owned(),
        kind: "platform",
        origin: "core",
        version: PLATFORM.version.to_owned(),
        maturity: PLATFORM.maturity.as_str(),
        schema: Some(kampus_db::MODULE),
    };
    std::iter::once(platform)
        .chain(build().into_iter().map(|c| ComponentRecord {
            name: c.name(),
            kind: c.kind,
            origin: if c.external { "external" } else { "core" },
            version: c.manifest.version.to_owned(),
            maturity: c.manifest.maturity.as_str(),
            schema: c.schema,
        }))
        .collect()
}
