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
use kampus_access::PrincipalId;
use kampus_common::{AdapterSettings, ErrorKind};
use kampus_files::service::serve::Part;

use crate::world::{png, who, world};

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
        .open_upload(&w.ana, ("testing.docs", w.campus), "a.txt", 4097, None)
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
    let (served, _, data) = f
        .content(file.id, Part::Original, expires, signature)
        .await
        .unwrap();
    assert_eq!(
        (served.name.as_str(), &data[..]),
        ("notes.txt", &b"the notes"[..])
    );
    let changed = f
        .content(file.id, Part::Original, expires + 1, signature)
        .await
        .unwrap_err();
    assert_eq!(changed.code(), "files.link_invalid");
    let past = f
        .content(file.id, Part::Original, 1_000, signature)
        .await
        .unwrap_err();
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

/// A scanner that finds the test file.
#[derive(Debug)]
struct FindsTheTestFile;

#[async_trait::async_trait]
impl kampus_scan::Scanner for FindsTheTestFile {
    async fn scan(&self, content: Bytes) -> kampus_common::Result<kampus_scan::Verdict> {
        let found = content
            .windows(kampus_scan::EICAR.len())
            .any(|w| w == kampus_scan::EICAR);
        Ok(if found {
            kampus_scan::Verdict::Infected("Eicar-Test-Signature".to_owned())
        } else {
            kampus_scan::Verdict::Clean
        })
    }

    async fn check(&self) -> kampus_common::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn infected_content_is_never_kept_and_clean_content_is_marked_clean() {
    let w = world().await;
    let unscanned = w.upload("before.txt", b"kept before").await;
    assert_eq!(
        w.count("select count(*) from {s}.blob where scan_status = 'unscanned'")
            .await,
        1
    );
    let f = w.files.clone().with_scanner(Some(Arc::new(FindsTheTestFile)));
    let upload = f
        .open_upload(
            &w.ana,
            ("testing.docs", w.campus),
            "eicar.txt",
            68,
            Some("text/plain"),
        )
        .await
        .unwrap();
    let infected = f
        .send_upload(&w.ana, upload.id, Bytes::from_static(kampus_scan::EICAR))
        .await
        .unwrap_err();
    assert_eq!(infected.code(), "files.infected");
    assert_eq!(
        w.count("select count(*) from {s}.blob").await,
        1,
        "nothing infected is stored"
    );
    let again = f
        .open_upload(
            &w.ana,
            ("testing.docs", w.campus),
            "again.txt",
            11,
            Some("text/plain"),
        )
        .await
        .unwrap();
    f.send_upload(&w.ana, again.id, Bytes::from_static(b"kept before"))
        .await
        .unwrap();
    assert_eq!(
        w.count("select count(*) from {s}.blob where scan_status = 'clean'")
            .await,
        1
    );
    assert!(!unscanned.attached);
    w.drop().await;
}

#[tokio::test]
async fn a_thumbnail_is_made_once_by_a_job_and_read_through_its_own_link() {
    let w = world().await;
    let f = &w.files;
    let picture = w.upload_owned("wide.png", png(600, 300)).await;
    let queued = f
        .conversion(who(&w.ana), picture.id, "thumbnail", &w.services)
        .await
        .unwrap();
    assert_eq!((queued.state.as_str(), queued.url), ("queued", None));
    assert_eq!(w.count("select count(*) from {s}.conversion").await, 1);
    f.conversion(who(&w.ana), picture.id, "thumbnail", &w.services)
        .await
        .unwrap();
    assert_eq!(
        w.count("select count(*) from {s}.conversion").await,
        1,
        "asked twice, made once"
    );
    w.run_conversions().await;
    let done = f
        .conversion(who(&w.ana), picture.id, "thumbnail", &w.services)
        .await
        .unwrap();
    assert_eq!(
        (done.state.as_str(), done.media_type.as_deref()),
        ("done", Some("image/png"))
    );
    let url = done.url.unwrap();
    let query = url.split_once('?').unwrap().1;
    let expires: i64 = query
        .split('&')
        .next()
        .unwrap()
        .trim_start_matches("expires=")
        .parse()
        .unwrap();
    let signature = query
        .split("signature=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();
    let (_, media_type, data) = f
        .content(picture.id, Part::Conversion("thumbnail"), expires, signature)
        .await
        .unwrap();
    assert_eq!(media_type, "image/png");
    let thumbnail = image::load_from_memory(&data).unwrap();
    assert_eq!((thumbnail.width(), thumbnail.height()), (256, 128));
    let original = f
        .content(picture.id, Part::Original, expires, signature)
        .await
        .unwrap_err();
    assert_eq!(
        original.code(),
        "files.link_invalid",
        "a thumbnail link never reads the original"
    );
    let text = w.upload("notes.txt", b"not an image").await;
    let unsupported = f
        .conversion(who(&w.ana), text.id, "thumbnail", &w.services)
        .await
        .unwrap_err();
    assert_eq!(unsupported.code(), "files.conversion_unsupported");
    let broken = w
        .upload_owned("broken.png", b"\x89PNG\r\n\x1a\nnot really".to_vec())
        .await;
    f.conversion(who(&w.ana), broken.id, "thumbnail", &w.services)
        .await
        .unwrap();
    w.run_conversions().await;
    let failed = f
        .conversion(who(&w.ana), broken.id, "thumbnail", &w.services)
        .await
        .unwrap();
    assert_eq!(failed.state, "failed");
    assert!(failed.error.unwrap().starts_with("unreadable image"));
    w.drop().await;
}

#[tokio::test]
async fn content_is_copied_to_another_store_verified_and_only_once() {
    let w = world().await;
    w.upload("a.txt", b"first content").await;
    w.upload("b.txt", b"second content").await;
    let other = tempfile::tempdir().unwrap();
    let target = kampus_storage::open(
        &[kampus_storage_opendal::ADAPTER],
        AdapterSettings {
            url: format!("file://{}", other.path().to_str().unwrap()),
            ..AdapterSettings::default()
        },
    )
    .await
    .unwrap();
    let first = w.files.copy_to(target.as_ref()).await.unwrap();
    assert_eq!((first.copied, first.present), (2, 0));
    let again = w.files.copy_to(target.as_ref()).await.unwrap();
    assert_eq!((again.copied, again.present), (0, 2));
    let key = kampus_storage::StorageKey::for_hash(&kampus_storage::sha256(b"first content"));
    assert_eq!(&target.get(&key).await.unwrap()[..], b"first content");
    w.drop().await;
}
