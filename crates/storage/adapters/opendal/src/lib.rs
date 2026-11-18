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

use std::ops::Range;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use kampus_common::{Error, Result};
use kampus_storage::{Capabilities, ObjectInfo, Storage, StorageKey, verify};
use opendal::{ErrorKind as DalKind, Operator, services};

/// The settings of an S3-compatible store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3Config {
    /// The endpoint, for stores other than AWS (`http://seaweedfs:8333`); `None` for AWS.
    pub endpoint: Option<String>,
    /// The region (`us-east-1` for most S3-compatible stores).
    pub region: String,
    /// The bucket.
    pub bucket: String,
    /// A prefix inside the bucket, so several installations can share it.
    pub root: String,
    /// The access key.
    pub access_key_id: String,
    /// The secret key.
    pub secret_access_key: String,
}

/// A file store on OpenDAL.
#[derive(Debug, Clone)]
pub struct OpendalStorage {
    /// The OpenDAL operator of the backend.
    operator: Operator,
}

impl OpendalStorage {
    /// A store on the local disk under `root`; blobs are written to a temporary directory and moved into
    /// place once complete.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the backend cannot be configured.
    pub fn local(root: &str) -> Result<Self> {
        opendal::install_default();
        let builder = services::Fs::default()
            .root(root)
            .atomic_write_dir(&format!("{}/.incoming", root.trim_end_matches('/')));
        Ok(Self {
            operator: Operator::new(builder).map_err(unavailable)?,
        })
    }

    /// A store in an S3-compatible bucket.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the backend cannot be configured.
    pub fn s3(config: &S3Config) -> Result<Self> {
        opendal::install_default();
        let mut builder = services::S3::default()
            .bucket(&config.bucket)
            .region(&config.region)
            .root(&config.root)
            .access_key_id(&config.access_key_id)
            .secret_access_key(&config.secret_access_key);
        if let Some(endpoint) = &config.endpoint {
            builder = builder.endpoint(endpoint);
        }
        Ok(Self {
            operator: Operator::new(builder).map_err(unavailable)?,
        })
    }
}

#[async_trait]
impl Storage for OpendalStorage {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            presign_read: self.operator.info().capability().presign_read,
        }
    }

    async fn put(&self, key: &StorageKey, data: Bytes, sha256: [u8; 32]) -> Result<()> {
        verify(key, &data, &sha256)?;
        self.operator
            .write(key.as_str(), data)
            .await
            .map(drop)
            .map_err(unavailable)
    }

    async fn get(&self, key: &StorageKey) -> Result<Bytes> {
        self.operator
            .read(key.as_str())
            .await
            .map(|buffer| buffer.to_bytes())
            .map_err(|error| failure(key, error))
    }

    async fn get_range(&self, key: &StorageKey, range: Range<u64>) -> Result<Bytes> {
        self.operator
            .read_with(key.as_str())
            .range(range)
            .await
            .map(|buffer| buffer.to_bytes())
            .map_err(|error| failure(key, error))
    }

    async fn head(&self, key: &StorageKey) -> Result<Option<ObjectInfo>> {
        match self.operator.stat(key.as_str()).await {
            Ok(meta) => Ok(Some(ObjectInfo {
                size: meta.content_length(),
            })),
            Err(error) if error.kind() == DalKind::NotFound => Ok(None),
            Err(error) => Err(unavailable(error)),
        }
    }

    async fn delete(&self, key: &StorageKey) -> Result<()> {
        self.operator.delete(key.as_str()).await.map_err(unavailable)
    }

    async fn copy(&self, from: &StorageKey, to: &StorageKey) -> Result<()> {
        if self.operator.info().capability().copy {
            return self
                .operator
                .copy(from.as_str(), to.as_str())
                .await
                .map(drop)
                .map_err(|error| failure(from, error));
        }
        let data = self.get(from).await?;
        self.operator
            .write(to.as_str(), data)
            .await
            .map(drop)
            .map_err(unavailable)
    }

    async fn presign_read(&self, key: &StorageKey, ttl: Duration) -> Result<Option<String>> {
        if !self.capabilities().presign_read {
            return Ok(None);
        }
        let request = self
            .operator
            .presign_read(key.as_str(), ttl)
            .await
            .map_err(unavailable)?;
        Ok(Some(request.uri().to_string()))
    }

    async fn check(&self) -> Result<()> {
        self.operator.check().await.map_err(unavailable)
    }
}

/// Maps a backend failure about a blob: missing blobs are `not_found`, everything else `unavailable`.
fn failure(key: &StorageKey, error: opendal::Error) -> Error {
    if error.kind() == DalKind::NotFound {
        Error::not_found("storage.not_found").with_param("key", key.as_str())
    } else {
        unavailable(error)
    }
}

/// Wraps a backend failure as `unavailable`.
fn unavailable(error: opendal::Error) -> Error {
    Error::unavailable("storage.unavailable").with_source(error)
}

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
