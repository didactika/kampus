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

//! Opens the infrastructure ports of a process with the adapters of this build: the scheme of
//! each URL in the configuration chooses the adapter.

use std::sync::Arc;
use std::time::Duration;

use kampus_cache::{Cache, Layered};
use kampus_common::{AdapterSettings, Result};
use kampus_storage::Storage;

use crate::config::Config;
use crate::wiring::adapters;

/// How long a value read from the shared cache stays in the in-process one.
const LOCAL_TTL: Duration = Duration::from_secs(30);

/// The ports a process uses besides the database.
#[derive(Debug, Clone)]
pub struct Ports {
    /// The file storage, for roles that handle files.
    pub storage: Option<Arc<dyn Storage>>,
    /// The cache: in-process, in front of the shared one when there is one.
    pub cache: Arc<dyn Cache>,
    /// The malware scanner, when one is configured.
    pub scanner: Option<Arc<dyn kampus_scan::Scanner>>,
    /// The mail server, when one is configured.
    pub mailer: Option<Arc<dyn kampus_mail::Mailer>>,
    /// The push services, when they are configured.
    pub pusher: Option<Arc<dyn kampus_push::Pusher>>,
}

impl Ports {
    /// Opens the ports the configuration names.
    ///
    /// # Errors
    ///
    /// A `validation` error when no adapter of this build serves a URL, or the adapter's error.
    pub async fn open(config: &Config) -> Result<Self> {
        let prefix = config.database_prefix.as_str().to_owned();
        let storage = match &config.storage_url {
            Some(url) => {
                let settings = AdapterSettings {
                    url: url.clone(),
                    prefix: prefix.clone(),
                    options: config.storage_options.clone(),
                };
                Some(kampus_storage::open(adapters::STORAGE, settings).await?)
            }
            None => None,
        };
        let scanner = match &config.scan_url {
            Some(url) => Some(
                kampus_scan::open(
                    adapters::SCAN,
                    AdapterSettings {
                        url: url.clone(),
                        prefix: prefix.clone(),
                        ..AdapterSettings::default()
                    },
                )
                .await?,
            ),
            None => None,
        };
        let mailer = match &config.mail_url {
            Some(url) => Some(
                kampus_mail::open(
                    adapters::MAIL,
                    AdapterSettings {
                        url: url.clone(),
                        prefix: prefix.clone(),
                        options: config.mail_options.clone(),
                    },
                )
                .await?,
            ),
            None => None,
        };
        let pusher = match &config.push_url {
            Some(url) => Some(
                kampus_push::open(
                    adapters::PUSH,
                    AdapterSettings {
                        url: url.clone(),
                        prefix: prefix.clone(),
                        options: config.push_options.clone(),
                    },
                )
                .await?,
            ),
            None => None,
        };
        let local = kampus_cache::open(
            adapters::CACHE,
            AdapterSettings {
                url: "memory://".into(),
                prefix: prefix.clone(),
                ..AdapterSettings::default()
            },
        )
        .await?;
        let cache = match &config.cache_url {
            Some(url) => {
                let shared = kampus_cache::open(
                    adapters::CACHE,
                    AdapterSettings {
                        url: url.clone(),
                        prefix,
                        ..AdapterSettings::default()
                    },
                )
                .await?;
                Arc::new(Layered::new(local, shared, LOCAL_TTL)) as Arc<dyn Cache>
            }
            None => local,
        };
        Ok(Self {
            storage,
            cache,
            scanner,
            mailer,
            pusher,
        })
    }

    /// The ports of a test: a store in a directory and an in-process cache.
    ///
    /// # Errors
    ///
    /// The adapter's error when the directory cannot be used.
    pub async fn for_tests(directory: &str) -> Result<Self> {
        let settings = AdapterSettings {
            url: format!("file://{directory}"),
            ..AdapterSettings::default()
        };
        let storage = kampus_storage::open(adapters::STORAGE, settings).await?;
        let cache = kampus_cache::open(
            adapters::CACHE,
            AdapterSettings {
                url: "memory://".into(),
                ..AdapterSettings::default()
            },
        )
        .await?;
        Ok(Self {
            storage: Some(storage),
            cache,
            scanner: None,
            mailer: None,
            pusher: None,
        })
    }
}
