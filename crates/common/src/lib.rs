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

//! Foundations shared by every Kampus module, as fixed by the platform foundations: the error model,
//! typed identifiers, instants and the clock, cursor pagination, request identifiers, what every component
//! says about itself, and the platform version.
//!
//! This crate depends only on `kampus-macros`.

extern crate self as kampus_common;

pub mod adapter;
pub mod clock;
pub mod component;
pub mod error;
pub mod id;
pub mod page;
pub mod platform;
pub mod request;

pub use adapter::AdapterSettings;
pub use clock::{Clock, FixedClock, SystemClock, Timestamp};
pub use error::{Error, ErrorKind, FieldError, Result};
pub use kampus_macros::manifest;
pub use page::{Cursor, Page, PageRequest};
pub use request::RequestId;

pub use platform::PLATFORM;

/// The version of the Kampus platform this build belongs to, from `version.toml`.
pub const PLATFORM_VERSION: &str = PLATFORM.version;

/// Re-exports used by the macros of this crate, so that crates using them need no extra dependency.
#[doc(hidden)]
pub mod __private {
    pub use serde;
    pub use uuid;
}
