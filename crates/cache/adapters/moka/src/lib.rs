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
// @package   kampus-cache-moka
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The in-process cache adapter on moka: the local level of the cache, and the whole cache of an
//! installation that runs a single instance.

mod store;

use kampus_cache::Cache;

pub use store::MokaCache;

/// The registration of this adapter: `memory://` gives an in-process cache, the local level of
/// every installation; the option `max_entries` bounds it (100 000 by default).
pub const ADAPTER: kampus_cache::Adapter = kampus_cache::Adapter {
    manifest: kampus_common::manifest!(),
    name: "moka",
    schemes: &["memory"],
    open: |settings| {
        Box::pin(async move {
            let max_entries = settings
                .option("max_entries")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000);
            Ok(std::sync::Arc::new(MokaCache::new(max_entries)) as std::sync::Arc<dyn Cache>)
        })
    },
};
