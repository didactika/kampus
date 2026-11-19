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

//! Tests of `kampus-common`: component check.

use kampus_common::component::{Manifest, Maturity, Requirement};
use kampus_common::component::{Registered, check};

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
        date: None,
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
