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
// @package   kampus-mail
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! How adapters register in a build and how the configured one is opened.

use std::sync::Arc;

use kampus_common::Result;

use crate::Mailer;

/// What opening an adapter returns.
pub type Opening = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Arc<dyn Mailer>>> + Send>>;

/// The registration of a mail adapter in a build.
#[derive(Debug, Clone, Copy)]
pub struct Adapter {
    /// What it is, its version and what it requires.
    pub manifest: kampus_common::component::Manifest,
    /// Its name.
    pub name: &'static str,
    /// The URL schemes it opens, such as `smtp`.
    pub schemes: &'static [&'static str],
    /// Opens it with the settings of the configuration.
    pub open: fn(kampus_common::AdapterSettings) -> Opening,
}

/// Opens the adapter of the build that serves the scheme of the configured URL.
///
/// # Errors
///
/// `validation` when no adapter serves the scheme, and the errors of the adapter.
pub async fn open(adapters: &[Adapter], settings: kampus_common::AdapterSettings) -> Result<Arc<dyn Mailer>> {
    let adapter = kampus_common::adapter::choose(adapters, |a| a.schemes, &settings, "mail")?;
    (adapter.open)(settings).await
}
