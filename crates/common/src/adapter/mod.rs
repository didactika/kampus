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

//! What every infrastructure port shares about its adapters: the settings an adapter is opened
//! with, and how the scheme of its URL chooses it.

use std::collections::BTreeMap;

use crate::{Error, FieldError, Result};

/// The settings an adapter is opened with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdapterSettings {
    /// The URL that chose the adapter (`redis://cache:6379`, `s3://bucket/prefix`, `file:///var/lib/kampus`).
    pub url: String,
    /// The installation prefix, for adapters that share a backend between installations.
    pub prefix: String,
    /// The adapter's own options, from `KAMPUS_<PORT>_<OPTION>` variables (`s3_endpoint` …).
    pub options: BTreeMap<String, String>,
}

impl AdapterSettings {
    /// An option of the adapter, if set.
    #[must_use]
    pub fn option(&self, name: &str) -> Option<&str> {
        self.options.get(name).map(String::as_str)
    }

    /// The scheme of the URL, in lower case: `redis` for `redis://cache:6379`.
    ///
    /// # Errors
    ///
    /// A `validation` error when the URL has no scheme.
    pub fn scheme(&self) -> Result<String> {
        self.url
            .split_once("://")
            .map(|(scheme, _)| scheme.to_ascii_lowercase())
            .filter(|scheme| !scheme.is_empty())
            .ok_or_else(|| {
                Error::validation("common.adapter_url", vec![FieldError::new("url", "scheme")])
                    .with_param("url", self.url.as_str())
            })
    }

    /// What follows the scheme: `cache:6379` for `redis://cache:6379`.
    #[must_use]
    pub fn location(&self) -> &str {
        self.url.split_once("://").map_or("", |(_, rest)| rest)
    }
}

/// Picks the adapter that serves a scheme among those of a build.
///
/// # Errors
///
/// A `validation` error naming the schemes of the build when none serves it.
pub fn choose<'a, A>(
    adapters: &'a [A],
    schemes: impl Fn(&A) -> &'static [&'static str],
    settings: &AdapterSettings,
    port: &str,
) -> Result<&'a A> {
    let scheme = settings.scheme()?;
    adapters
        .iter()
        .find(|adapter| schemes(adapter).contains(&scheme.as_str()))
        .ok_or_else(|| {
            let known: Vec<&str> = adapters.iter().flat_map(|a| schemes(a).iter().copied()).collect();
            Error::validation(
                format!("{port}.adapter_unknown"),
                vec![FieldError::new("url", "scheme")],
            )
            .with_param("scheme", scheme)
            .with_param("known", known.join(", "))
        })
}

#[cfg(test)]
mod tests {
    use super::{AdapterSettings, choose};

    #[test]
    fn the_scheme_chooses_the_adapter() {
        let adapters = [("memory", &["memory"][..]), ("redis", &["redis", "valkey"][..])];
        let settings = AdapterSettings {
            url: "Valkey://cache:6379".into(),
            ..AdapterSettings::default()
        };
        assert_eq!(settings.location(), "cache:6379");
        assert_eq!(choose(&adapters, |a| a.1, &settings, "cache").unwrap().0, "redis");
        let unknown = AdapterSettings {
            url: "memcached://x".into(),
            ..AdapterSettings::default()
        };
        let error = choose(&adapters, |a| a.1, &unknown, "cache").unwrap_err();
        assert_eq!(error.code(), "cache.adapter_unknown");
        assert!(error.to_string().contains("known=memory, redis, valkey"));
        assert!(AdapterSettings::default().scheme().is_err());
    }
}
