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
// @package   kampus-storage-opendal
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! File storage adapters on Apache OpenDAL (DEC-009): local disk and S3-compatible stores now, and every
//! backend OpenDAL supports later, behind the [`Storage`] port.

mod store;

use kampus_common::{Error, Result};
use kampus_storage::Storage;

pub use store::{OpendalStorage, S3Config};

/// The registration of this adapter (DEC-086): `file:///path` stores on the local disk; `s3://bucket/prefix`
/// in an S3-compatible bucket, with the options `s3_endpoint`, `s3_region`, `s3_access_key_id` and
/// `s3_secret_access_key`.
pub const ADAPTER: kampus_storage::Adapter = kampus_storage::Adapter {
    name: "opendal",
    schemes: &["file", "s3"],
    open: |settings| {
        Box::pin(
            async move { open(&settings).map(|s| std::sync::Arc::new(s) as std::sync::Arc<dyn Storage>) },
        )
    },
};

/// Builds the store a URL names.
fn open(settings: &kampus_common::AdapterSettings) -> Result<OpendalStorage> {
    let location = settings.location();
    if settings.scheme()? == "file" {
        return OpendalStorage::local(location);
    }
    let (bucket, root) = location.split_once('/').unwrap_or((location, ""));
    let option = |name: &str| settings.option(name).map(str::to_owned);
    let missing = |name: &str| {
        kampus_common::FieldError::new(
            format!("KAMPUS_STORAGE_{}", name.to_ascii_uppercase()),
            "required",
        )
    };
    let (Some(access_key_id), Some(secret_access_key)) =
        (option("s3_access_key_id"), option("s3_secret_access_key"))
    else {
        return Err(Error::validation(
            "storage.s3_credentials",
            vec![missing("s3_access_key_id"), missing("s3_secret_access_key")],
        ));
    };
    OpendalStorage::s3(&S3Config {
        endpoint: option("s3_endpoint"),
        region: option("s3_region").unwrap_or_else(|| "us-east-1".into()),
        bucket: bucket.to_owned(),
        root: format!("/{root}"),
        access_key_id,
        secret_access_key,
    })
}
