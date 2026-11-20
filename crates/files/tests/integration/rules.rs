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

//! One test per behaviour of files, against PostgreSQL and a local store: the upload pipeline, content
//! stored once, who reads a file before and after it is attached, signed links, and the purge.

use std::sync::Arc;

use bytes::Bytes;
use kampus_access::{
    Access, Actor, PermissionDeclaration, PermissionKind, PrincipalId, ScopeId, ScopeKind, Value,
};
use kampus_common::{AdapterSettings, ErrorKind, SystemClock};
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
    max_size: 64,
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
struct World {
    /// The service.
    files: Files,
    /// The database.
    db: Database,
    /// The campus.
    campus: ScopeId,
    /// Someone who may upload.
    ana: Actor,
    /// Someone else, signed in.
    bea: Actor,
    /// The services, empty: the area of these tests needs none.
    services: ServiceContext,
    /// Keeps the store.
    _store: tempfile::TempDir,
}

/// Builds the world.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("f{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    let schemas: Vec<_> = [
        &kampus_events::MODULE,
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
    let store = tempfile::tempdir().unwrap();
    let storage = kampus_storage::open(
        &[kampus_storage_opendal::ADAPTER],
        AdapterSettings {
            url: format!("file://{}", store.path().to_str().unwrap()),
            ..AdapterSettings::default()
        },
    )
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
    async fn drop(self) {
        for module in ["files", "access", "events", "db"] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
    }

    /// Uploads content as Ana through the whole pipeline.
    async fn upload(&self, name: &str, content: &'static [u8]) -> File {
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
    async fn attach(&self, file: &File) -> kampus_common::Result<File> {
        let mut tx = self.db.begin().await.unwrap();
        let attached = self
            .files
            .attach_in(&mut tx, file.id, "testing.docs", None)
            .await?;
        tx.commit().await.unwrap();
        Ok(attached)
    }

    /// A number of a table of files.
    async fn count(&self, query: &str) -> i64 {
        sqlx::query_scalar(sqlx::AssertSqlSafe(
            query.replace("{s}", &self.db.prefix().schema("files")),
        ))
        .fetch_one(self.db.pool())
        .await
        .unwrap()
    }
}

/// The principal of an actor.
fn who(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) => Some(p.as_uuid()),
        _ => None,
    }
}

#[tokio::test]
async fn an_upload_keeps_to_its_area_and_the_content_decides_the_type() {
    let w = world().await;
    let f = &w.files;
    let denied = f
        .open_upload(&w.bea, ("testing.docs", w.campus), "a.txt", 3, None)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let large = f
        .open_upload(&w.ana, ("testing.docs", w.campus), "a.txt", 65, None)
        .await
        .unwrap_err();
    assert_eq!(large.code(), "files.upload_invalid");
    let unknown = f
        .open_upload(&w.ana, ("testing.none", w.campus), "a", 1, None)
        .await
        .unwrap_err();
    assert_eq!(unknown.code(), "files.area_unknown");
    let bad_name = f
        .open_upload(&w.ana, ("testing.docs", w.campus), "../etc/passwd", 3, None)
        .await
        .unwrap_err();
    assert_eq!(bad_name.code(), "files.upload_invalid");
    let upload = f
        .open_upload(
            &w.ana,
            ("testing.docs", w.campus),
            "page.txt",
            64,
            Some("text/plain"),
        )
        .await
        .unwrap();
    let pdf = f
        .send_upload(
            &w.ana,
            upload.id,
            Bytes::from_static(b"%PDF-1.7 not allowed here"),
        )
        .await
        .unwrap_err();
    assert_eq!(pdf.code(), "files.type_not_allowed");
    let other = f
        .send_upload(&w.bea, upload.id, Bytes::from_static(b"hello"))
        .await
        .unwrap_err();
    assert_eq!(other.kind(), ErrorKind::NotFound);
    let empty = f.complete_upload(&w.ana, upload.id).await.unwrap_err();
    assert_eq!(empty.code(), "files.upload_empty");
    f.send_upload(&w.ana, upload.id, Bytes::from_static(b"hello"))
        .await
        .unwrap();
    let file = f.complete_upload(&w.ana, upload.id).await.unwrap();
    assert_eq!(
        (file.media_type.as_str(), file.size, file.attached),
        ("text/plain", 5, false)
    );
    let again = f.complete_upload(&w.ana, upload.id).await.unwrap_err();
    assert_eq!(again.code(), "files.upload_completed");
    w.drop().await;
}

#[tokio::test]
async fn content_is_stored_once_and_read_by_its_uploader_until_it_is_attached() {
    let w = world().await;
    let f = &w.files;
    let first = w.upload("a.txt", b"same words").await;
    let second = w.upload("b.txt", b"same words").await;
    assert_eq!(w.count("select count(*) from {s}.blob").await, 1);
    assert_eq!(w.count("select ref_count::bigint from {s}.blob").await, 2);
    assert_eq!(w.count("select count(*) from {s}.trash").await, 0);
    f.file(who(&w.ana), first.id, &w.services).await.unwrap();
    let hidden = f.file(who(&w.bea), first.id, &w.services).await.unwrap_err();
    assert_eq!(hidden.kind(), ErrorKind::NotFound);
    let mut tx = w.db.begin().await.unwrap();
    let wrong = f
        .attach_in(&mut tx, first.id, "testing.other", None)
        .await
        .unwrap_err();
    assert_eq!(wrong.code(), "files.area_mismatch");
    let not_hers = f
        .attach_in(&mut tx, first.id, "testing.docs", who(&w.bea))
        .await
        .unwrap_err();
    assert_eq!(not_hers.kind(), ErrorKind::NotFound);
    f.attach_in(&mut tx, first.id, "testing.docs", who(&w.ana))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    f.file(who(&w.bea), first.id, &w.services).await.unwrap();
    assert_eq!(
        f.file(None, first.id, &w.services).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    let mut tx = w.db.begin().await.unwrap();
    f.detach_in(&mut tx, first.id).await.unwrap();
    f.detach_in(&mut tx, second.id).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(w.count("select ref_count::bigint from {s}.blob").await, 0);
    assert_eq!(w.count("select count(*) from {s}.trash").await, 1);
    w.drop().await;
}

#[tokio::test]
async fn a_signed_link_reads_one_file_until_it_expires() {
    let w = world().await;
    let f = &w.files;
    let file = w.upload("notes.txt", b"the notes").await;
    let denied = f.signed_url(who(&w.bea), file.id, &w.services).await.unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::NotFound);
    let link = f.signed_url(who(&w.ana), file.id, &w.services).await.unwrap();
    let query = link.url.split_once('?').unwrap().1;
    let expires: i64 = query
        .split('&')
        .next()
        .unwrap()
        .trim_start_matches("expires=")
        .parse()
        .unwrap();
    let signature = query.split("signature=").nth(1).unwrap();
    assert!(
        link.url
            .starts_with(&format!("http://kampus.test/api/v1/files/content/{}", file.id))
    );
    let (served, data) = f.content(file.id, expires, signature).await.unwrap();
    assert_eq!(
        (served.name.as_str(), &data[..]),
        ("notes.txt", &b"the notes"[..])
    );
    let changed = f.content(file.id, expires + 1, signature).await.unwrap_err();
    assert_eq!(changed.code(), "files.link_invalid");
    let past = f.content(file.id, 1_000, signature).await.unwrap_err();
    assert_eq!(past.code(), "files.link_invalid");
    w.drop().await;
}

#[tokio::test]
async fn the_purge_removes_what_nobody_attached_and_content_past_its_trash() {
    let w = world().await;
    let f = &w.files;
    let kept = w.upload("kept.txt", b"kept").await;
    let mut tx = w.db.begin().await.unwrap();
    f.attach_in(&mut tx, kept.id, "testing.docs", None).await.unwrap();
    tx.commit().await.unwrap();
    w.upload("left.txt", b"left behind").await;
    let schema = w.db.prefix().schema("files");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {schema}.file set created_at = now() - interval '2 days';
         update {schema}.upload set expires_at = now() - interval '1 hour'"
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    let first = f.purge().await.unwrap();
    assert_eq!((first.uploads, first.files, first.blobs), (0, 1, 0));
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {schema}.trash set since = now() - interval '8 days'"
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    let second = f.purge().await.unwrap();
    assert_eq!(second.blobs, 1);
    assert_eq!(w.count("select count(*) from {s}.blob").await, 1);
    f.file(who(&w.bea), kept.id, &w.services).await.unwrap();
    w.drop().await;
}

#[tokio::test]
async fn quotas_refuse_what_does_not_fit_and_count_only_attached_files() {
    let w = world().await;
    let f = &w.files;
    let quota = f.set_quota(&w.ana, w.campus, Some(20)).await.unwrap();
    assert_eq!((quota.limit_bytes, quota.used_bytes), (Some(20), 0));
    let first = w.upload("one.txt", b"ten bytes!").await;
    assert_eq!(
        f.quota(&w.ana, w.campus).await.unwrap().used_bytes,
        0,
        "only attached files count"
    );
    w.attach(&first).await.unwrap();
    assert_eq!(f.quota(&w.ana, w.campus).await.unwrap().used_bytes, 10);
    let too_big = f
        .open_upload(&w.ana, ("testing.docs", w.campus), "big.txt", 11, None)
        .await
        .unwrap_err();
    assert_eq!(too_big.code(), "files.quota_exceeded");
    let second = w.upload("two.txt", b"eight b!").await;
    let third = w.upload("three.txt", b"eight c!").await;
    w.attach(&second).await.unwrap();
    let refused = w.attach(&third).await.unwrap_err();
    assert_eq!(refused.code(), "files.quota_exceeded");
    assert_eq!(f.quota(&w.ana, w.campus).await.unwrap().used_bytes, 18);
    let ana = PrincipalId::from_uuid(who(&w.ana).unwrap());
    let personal = f.person_quota(&w.ana, ana).await.unwrap();
    assert_eq!(personal.used_bytes, 18);
    f.set_person_quota(&w.ana, ana, Some(19)).await.unwrap();
    let person_full = f
        .open_upload(&w.ana, ("testing.docs", w.campus), "two.txt", 2, None)
        .await
        .unwrap_err();
    assert_eq!(person_full.code(), "files.quota_exceeded");
    let mut tx = w.db.begin().await.unwrap();
    f.detach_in(&mut tx, first.id).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(f.quota(&w.ana, w.campus).await.unwrap().used_bytes, 8);
    assert_eq!(f.person_quota(&w.ana, ana).await.unwrap().used_bytes, 8);
    let denied = f.set_quota(&w.bea, w.campus, None).await.unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let negative = f.set_quota(&w.ana, w.campus, Some(-1)).await.unwrap_err();
    assert_eq!(negative.code(), "files.quota_invalid");
    w.drop().await;
}
