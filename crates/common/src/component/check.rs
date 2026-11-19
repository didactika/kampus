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

//! The rules a build's set of components must follow before the platform installs, upgrades or starts.

use std::collections::BTreeMap;
use std::fmt;

use semver::{Version, VersionReq};

use super::Manifest;

/// A component as the build registers it.
#[derive(Debug, Clone, Copy)]
pub struct Registered<'a> {
    /// What it says about itself.
    pub manifest: &'a Manifest,
    /// Its type when it is a plugin or an adapter (`cache`, `activity`); `None` for a core module.
    pub plugin_type: Option<&'a str>,
    /// Whether it comes from outside the repository.
    pub external: bool,
}

/// Something wrong with a component, or missing from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The component, or its package when the name cannot be read.
    pub component: String,
    /// What is wrong.
    pub message: String,
    /// Whether it stops the platform; otherwise it is only shown.
    pub blocking: bool,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = if self.blocking { "" } else { " (warning)" };
        write!(f, "{}: {}{level}", self.component, self.message)
    }
}

/// Checks the components of a build against the Kampus release it runs and against each other: names,
/// versions, the releases each supports, what each requires, and the plugin types.
#[must_use]
pub fn check(components: &[Registered<'_>], plugin_types: &[&str], platform: &str) -> Vec<Problem> {
    let mut problems = Vec::new();
    let platform = Version::parse(platform).ok();
    let mut versions: BTreeMap<String, Option<Version>> = BTreeMap::new();
    for registered in components {
        let name = registered.manifest.component();
        if versions.contains_key(&name) {
            problems.push(problem(&name, "is registered twice"));
        }
        versions.insert(name, Version::parse(registered.manifest.version).ok());
    }
    for registered in components {
        problems.extend(check_one(registered, plugin_types, platform.as_ref(), &versions));
    }
    problems
}

/// The problems of one component.
fn check_one(
    registered: &Registered<'_>,
    plugin_types: &[&str],
    platform: Option<&Version>,
    versions: &BTreeMap<String, Option<Version>>,
) -> Vec<Problem> {
    let manifest = registered.manifest;
    let name = manifest.component();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let valid_name = manifest.package.starts_with("kampus-")
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid_name {
        errors.push(format!(
            "the package {} must be kampus-<name>, in lower-case letters, digits and hyphens",
            manifest.package
        ));
    }
    match registered.plugin_type {
        None if registered.external => errors.push(
            "only plugins and adapters come from outside the repository: name it <type>-<name>".to_owned(),
        ),
        None => {
            if let Some(kind) = plugin_types.iter().find(|t| name.starts_with(&format!("{t}_"))) {
                errors.push(format!(
                    "a core module may not start with the plugin type {kind}_"
                ));
            }
        }
        Some(kind) => {
            if !plugin_types.contains(&kind) {
                errors.push(format!("{kind} is not a plugin type of this build"));
            }
            if !name.starts_with(&format!("{kind}_")) {
                errors.push(format!(
                    "a component of type {kind} is named {kind}_<name> (package kampus-{kind}-<name>)"
                ));
            }
        }
    }
    if Version::parse(manifest.version).is_err() {
        errors.push(format!(
            "the version {} is not a semantic version",
            manifest.version
        ));
    }
    match manifest.kampus.map(|k| (k, VersionReq::parse(k))) {
        None if registered.external => {
            warnings.push("does not say which Kampus releases it supports".to_owned());
        }
        None => {}
        Some((kampus, Err(_))) => {
            errors.push(format!(
                "the supported releases {kampus} are not a version requirement"
            ));
        }
        Some((kampus, Ok(range))) => {
            if let Some(platform) = platform.filter(|p| !range.matches(p)) {
                errors.push(format!("supports Kampus {kampus}, not {platform}"));
            }
        }
    }
    for requirement in manifest.requires {
        let Ok(range) = VersionReq::parse(requirement.version) else {
            errors.push(format!(
                "the requirement {} {} is not a version requirement",
                requirement.component, requirement.version
            ));
            continue;
        };
        match versions.get(requirement.component) {
            None => errors.push(format!(
                "requires {}, which is not in this build",
                requirement.component
            )),
            Some(Some(version)) if !range.matches(version) => errors.push(format!(
                "requires {} {}, and the build has {version}",
                requirement.component, requirement.version
            )),
            Some(_) => {}
        }
    }
    if manifest.summary.trim().is_empty() {
        warnings.push("has no summary: write the package description".to_owned());
    }
    if manifest.license.trim().is_empty() {
        warnings.push("has no licence".to_owned());
    }
    match manifest.icon {
        Some(icon) if !icon.contains("<svg") => warnings.push("its icon is not an SVG document".to_owned()),
        None if registered.external => warnings.push("has no icon; a generic one is shown".to_owned()),
        _ => {}
    }
    let blocking = errors.into_iter().map(|m| (m, true));
    let shown = warnings.into_iter().map(|m| (m, false));
    blocking
        .chain(shown)
        .map(|(message, blocking)| Problem {
            component: name.clone(),
            message,
            blocking,
        })
        .collect()
}

/// A problem of a component, which stops the platform.
fn problem(component: &str, message: &str) -> Problem {
    Problem {
        component: component.to_owned(),
        message: message.to_owned(),
        blocking: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{Registered, check};
    use crate::component::{Manifest, Maturity, Requirement};

    /// A valid manifest of a package.
    const fn manifest(
        package: &'static str,
        version: &'static str,
        requires: &'static [Requirement],
    ) -> Manifest {
        Manifest {
            package,
            version,
            summary: "Does something",
            license: "AGPL-3.0-or-later",
            maturity: Maturity::Alpha,
            kampus: Some(">=0.1.0, <0.2.0"),
            requires,
            icon: Some("<svg/>"),
        }
    }

    /// The messages of a check.
    fn messages(components: &[Registered<'_>]) -> Vec<String> {
        check(components, &["cache", "activity"], "0.1.0")
            .into_iter()
            .map(|p| p.to_string())
            .collect()
    }

    #[test]
    fn a_consistent_build_passes() {
        let access = manifest("kampus-access", "0.1.0", &[]);
        let identity = manifest(
            "kampus-identity",
            "0.1.0",
            &[Requirement {
                component: "access",
                version: ">=0.1.0",
            }],
        );
        let redis = manifest("kampus-cache-redis", "1.2.0", &[]);
        let attendance = manifest("kampus-activity-attendance", "2.0.0", &[]);
        let core = |manifest| Registered {
            manifest,
            plugin_type: None,
            external: false,
        };
        assert_eq!(
            messages(&[
                core(&access),
                core(&identity),
                Registered {
                    manifest: &redis,
                    plugin_type: Some("cache"),
                    external: false,
                },
                Registered {
                    manifest: &attendance,
                    plugin_type: Some("activity"),
                    external: true,
                },
            ]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn names_versions_releases_and_requirements_are_checked() {
        let newer = Manifest {
            kampus: Some(">=0.2.0"),
            ..manifest("kampus-access", "0.1", &[])
        };
        let needy = manifest(
            "kampus-identity",
            "0.1.0",
            &[
                Requirement {
                    component: "access",
                    version: ">=0.2.0",
                },
                Requirement {
                    component: "files",
                    version: "*",
                },
            ],
        );
        let misnamed = manifest("kampus-attendance", "1.0.0", &[]);
        let unknown_type = manifest("kampus-quiz-essay", "1.0.0", &[]);
        let core_with_prefix = Manifest {
            icon: Some("not an icon"),
            ..manifest("kampus-cache-local", "1.0.0", &[])
        };
        let found = messages(&[
            Registered {
                manifest: &newer,
                plugin_type: None,
                external: false,
            },
            Registered {
                manifest: &needy,
                plugin_type: None,
                external: false,
            },
            Registered {
                manifest: &misnamed,
                plugin_type: Some("activity"),
                external: true,
            },
            Registered {
                manifest: &unknown_type,
                plugin_type: Some("quiz"),
                external: true,
            },
            Registered {
                manifest: &core_with_prefix,
                plugin_type: None,
                external: false,
            },
            Registered {
                manifest: &misnamed,
                plugin_type: None,
                external: true,
            },
        ]);
        for expected in [
            "access: the version 0.1 is not a semantic version",
            "access: supports Kampus >=0.2.0, not 0.1.0",
            "identity: requires files, which is not in this build",
            "attendance: a component of type activity is named activity_<name> (package kampus-activity-<name>)",
            "attendance: is registered twice",
            "quiz_essay: quiz is not a plugin type of this build",
            "cache_local: a core module may not start with the plugin type cache_",
            "cache_local: its icon is not an SVG document (warning)",
            "attendance: only plugins and adapters come from outside the repository: name it <type>-<name>",
        ] {
            assert!(
                found.iter().any(|m| m == expected),
                "missing {expected:?} in {found:#?}"
            );
        }
        assert!(
            !found.iter().any(|m| m.starts_with("identity: requires access")),
            "access 0.1 is unreadable, so its requirement cannot be judged: {found:#?}"
        );
    }
    #[test]
    fn optional_fields_never_block() {
        let plain = Manifest {
            maturity: Maturity::Stable,
            kampus: None,
            icon: None,
            ..manifest("kampus-access", "0.1.0", &[])
        };
        let core = Registered {
            manifest: &plain,
            plugin_type: None,
            external: false,
        };
        assert!(
            check(&[core], &[], "0.1.0").is_empty(),
            "a core component needs none of them"
        );
        let external = Manifest {
            kampus: None,
            icon: None,
            ..manifest("kampus-activity-attendance", "1.0.0", &[])
        };
        let found = check(
            &[Registered {
                manifest: &external,
                plugin_type: Some("activity"),
                external: true,
            }],
            &["activity"],
            "0.1.0",
        );
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|p| !p.blocking), "{found:#?}");
    }
}
