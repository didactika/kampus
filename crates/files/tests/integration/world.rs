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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The installation the tests of files run in: a campus, someone who uploads, someone else, and a store
//! on the local disk or in an S3-compatible bucket.

use std::collections::BTreeMap;
use std::sync::Arc;

use bytes::Bytes;
use kampus_access::{
    Access, Actor, PermissionDeclaration, PermissionKind, PrincipalId, ScopeId, ScopeKind, Value,
};
use kampus_common::{AdapterSettings, SystemClock};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_files::{File, Files};
use kampus_module::{FileAreaDeclaration, FileRead, Module, ReadFuture, ServiceContext};

/// The permission to upload in these tests.
const UPLOAD: &str = "testing.files.upload";

/// The permissions of these tests.
const PERMISSIONS: &[PermissionDeclaration] = &[PermissionDeclaration {
    code: UPLOAD,
    kind: PermissionKind::Write,
    risks: &[],
    lowest_kind: ScopeKind::Course,
}];

/// Whoever signed in reads the attached files of these tests.
fn signed_in(read: FileRead<'_>) -> ReadFuture<'_> {
    Box::pin(async move { Ok(read.principal.is_some()) })
}

/// The area of these tests.
const AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: "testing.docs",
    summary: "Documents",
    max_size: 4096,
    allowed_types: &["text/*", "image/png"],
    upload_permission: UPLOAD,
    readable: signed_in,
}];

/// A module that declares the area.
const TESTING: Module = Module {
    name: "testing",
    permissions: PERMISSIONS,
    templates: &[],
    file_areas: AREAS,
    ..kampus_files::MODULE
};

/// The modules whose areas files reads.
static MODULES: &[&Module] = &[&TESTING];

/// An installation with a campus, someone who uploads, someone else, and a local store.
pub(crate) struct World {
    /// The service.
    pub(crate) files: Files,
    /// The database.
    pub(crate) db: Database,
    /// The campus.
    pub(crate) campus: ScopeId,
    /// Someone who may upload.
    pub(crate) ana: Actor,
    /// Someone else, signed in.
    pub(crate) bea: Actor,
    /// The services, empty: the area of these tests needs none.
    pub(crate) services: ServiceContext,
    /// Keeps the store on the local disk, when it is there.
    _store: Option<tempfile::TempDir>,
}

/// Builds the world, with a store on the local disk.
pub(crate) async fn world() -> World {
    let store = tempfile::tempdir().unwrap();
    let settings = AdapterSettings {
        url: format!("file://{}", store.path().to_str().unwrap()),
        ..AdapterSettings::default()
    };
    world_on(settings, Some(store)).await
}

/// Builds the world, with a store in the test bucket (`KAMPUS_TEST_S3_ENDPOINT`), reached by browsers at
/// the same address when it signs links.
pub(crate) async fn world_on_s3(signs_links: bool) -> World {
    let endpoint = std::env::var("KAMPUS_TEST_S3_ENDPOINT").expect("KAMPUS_TEST_S3_ENDPOINT is set");
    let mut options = BTreeMap::from([
        ("s3_endpoint".to_owned(), endpoint.clone()),
        ("s3_access_key_id".to_owned(), "kampus".to_owned()),
        ("s3_secret_access_key".to_owned(), "kampus-secret".to_owned()),
    ]);
    if signs_links {
        options.insert("public_url".to_owned(), endpoint);
    }
    let settings = AdapterSettings {
        url: format!("s3://kampus-test/files-{}", uuid::Uuid::now_v7().simple()),
        options,
        ..AdapterSettings::default()
    };
    world_on(settings, None).await
}

/// Builds the world on a store.
async fn world_on(settings: AdapterSettings, store: Option<tempfile::TempDir>) -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("f{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let schemas: Vec<_> = [
        &kampus_events::MODULE,
        &kampus_tasks::MODULE,
        &kampus_access::MODULE,
        &kampus_files::MODULE,
    ]
    .iter()
    .map(|m| m.schema())
    .collect();
    Upgrader::new(&db, &schemas, "0.1.0").apply(true).await.unwrap();
    let access = Access::new(db.clone(), Arc::new(SystemClock));
    access
        .sync_modules(&[&kampus_access::MODULE, &kampus_files::MODULE, &TESTING])
        .await
        .unwrap();
    kampus_events::Events::new(db.clone())
        .sync_declarations(&[&kampus_files::MODULE])
        .await
        .unwrap();
    kampus_tasks::Tasks::new(db.clone())
        .sync_declarations(&[&kampus_files::MODULE], kampus_common::Clock::now(&SystemClock))
        .await
        .unwrap();
    let storage = kampus_storage::open(&[kampus_storage_opendal::ADAPTER], settings)
        .await
        .unwrap();
    let files = Files::new(
        db.clone(),
        access.clone(),
        Some(storage),
        [9; 32],
        "http://kampus.test/api/v1".into(),
    )
    .with_modules(MODULES);
    files.sync_declarations(&[&TESTING]).await.unwrap();
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    let roles = access.install_campus_roles(campus).await.unwrap();
    let (ana, bea) = (PrincipalId::generate(), PrincipalId::generate());
    for person in [ana, bea] {
        access.create_principal(person, campus).await.unwrap();
    }
    access
        .grant_for_module("setup", roles["campus_administrator"], campus, ana, None)
        .await
        .unwrap();
    access
        .set_role_value(
            &Actor::Principal(ana),
            roles["campus_administrator"],
            UPLOAD,
            Some(Value::Allow),
        )
        .await
        .unwrap();
    World {
        files,
        db,
        campus,
        ana: Actor::Principal(ana),
        bea: Actor::Principal(bea),
        services: ServiceContext::default(),
        _store: store,
    }
}

impl World {
    /// Drops the installation.
    pub(crate) async fn drop(self) {
        for module in ["files", "access", "tasks", "events", "db"] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
        self.db.close().await;
    }

    /// Uploads content as Ana through the whole pipeline.
    pub(crate) async fn upload(&self, name: &str, content: &'static [u8]) -> File {
        let size = i64::try_from(content.len()).unwrap();
        let upload = self
            .files
            .open_upload(
                &self.ana,
                ("testing.docs", self.campus),
                name,
                size,
                Some("text/plain"),
            )
            .await
            .unwrap();
        self.files
            .send_upload(&self.ana, upload.id, Bytes::from_static(content))
            .await
            .unwrap();
        self.files.complete_upload(&self.ana, upload.id).await.unwrap()
    }

    /// Attaches a file in a transaction of its own, kept only when it succeeds.
    pub(crate) async fn attach(&self, file: &File) -> kampus_common::Result<File> {
        let mut tx = self.db.begin().await.unwrap();
        let attached = self
            .files
            .attach_in(&mut tx, file.id, "testing.docs", None)
            .await?;
        tx.commit().await.unwrap();
        Ok(attached)
    }

    /// A number of a table of files.
    pub(crate) async fn count(&self, query: &str) -> i64 {
        sqlx::query_scalar(sqlx::AssertSqlSafe(
            query.replace("{s}", &self.db.prefix().schema("files")),
        ))
        .fetch_one(self.db.pool())
        .await
        .unwrap()
    }
}

/// The principal of an actor.
pub(crate) fn who(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) => Some(p.as_uuid()),
        _ => None,
    }
}

/// A PNG of a size, in one colour.
pub(crate) fn png(width: u32, height: u32) -> Vec<u8> {
    let picture = image::RgbImage::from_pixel(width, height, image::Rgb([30, 120, 200]));
    let mut out = std::io::Cursor::new(Vec::new());
    picture.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

impl World {
    /// Uploads content Ana owns, whatever it is.
    pub(crate) async fn upload_owned(&self, name: &str, content: Vec<u8>) -> File {
        let size = i64::try_from(content.len()).unwrap();
        let upload = self
            .files
            .open_upload(&self.ana, ("testing.docs", self.campus), name, size, None)
            .await
            .unwrap();
        self.files
            .send_upload(&self.ana, upload.id, Bytes::from(content))
            .await
            .unwrap();
        self.files.complete_upload(&self.ana, upload.id).await.unwrap()
    }

    /// Runs every queued conversion, as the job would.
    pub(crate) async fn run_conversions(&self) {
        let queued: Vec<uuid::Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "select id from {}.conversion where state = 'queued'",
            self.db.prefix().schema("files")
        )))
        .fetch_all(self.db.pool())
        .await
        .unwrap();
        for conversion in queued {
            self.files.convert(conversion).await.unwrap();
        }
    }
}
