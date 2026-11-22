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

//! Links signed by the store: for content that cannot run, at the public address of the store and signed
//! with the type, disposition and caching the API would set; the API's own link for active content and for
//! stores that sign none.

use crate::world::{png, who, world_on_s3};

/// The value of a header of a response.
fn header(response: &reqwest::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

#[tokio::test]
async fn content_that_cannot_run_is_read_from_the_store_signed_with_the_headers_of_the_api() {
    let endpoint = std::env::var("KAMPUS_TEST_S3_ENDPOINT").expect("KAMPUS_TEST_S3_ENDPOINT is set");
    let w = world_on_s3(true).await;
    let f = &w.files;
    let picture = png(40, 20);
    let file = w.upload_owned("the \"picture\".png", picture.clone()).await;
    let link = f.signed_url(who(&w.ana), file.id, &w.services).await.unwrap();
    assert!(
        link.url.starts_with(&endpoint),
        "{} is signed by the store",
        link.url
    );
    for asked in [
        "response-content-type=image/png",
        "response-content-disposition=inline%3B%20filename%3D%22the%20_picture_.png%22",
        "response-cache-control=private%2C%20max-age%3D300",
    ] {
        assert!(link.url.contains(asked), "{} carries {asked}", link.url);
    }
    let response = reqwest::get(&link.url).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(header(&response, "content-type").as_deref(), Some("image/png"));
    assert_eq!(&response.bytes().await.unwrap()[..], &picture[..]);
    f.conversion(who(&w.ana), file.id, "thumbnail", &w.services)
        .await
        .unwrap();
    w.run_conversions().await;
    let thumbnail = f
        .conversion(who(&w.ana), file.id, "thumbnail", &w.services)
        .await
        .unwrap()
        .url
        .unwrap();
    assert!(
        thumbnail.starts_with(&endpoint),
        "{thumbnail} is signed by the store"
    );
    let response = reqwest::get(&thumbnail).await.unwrap();
    assert_eq!(header(&response, "content-type").as_deref(), Some("image/png"));
    image::load_from_memory(&response.bytes().await.unwrap()).unwrap();
    w.drop().await;
}

#[tokio::test]
async fn active_content_and_stores_that_sign_no_links_are_read_through_the_api() {
    let api = "http://kampus.test/api/v1/files/content/";
    let w = world_on_s3(true).await;
    let page = w
        .upload("page.html", b"<!doctype html><script>alert(1)</script>")
        .await;
    assert_eq!(page.media_type, "text/html");
    let link = w
        .files
        .signed_url(who(&w.ana), page.id, &w.services)
        .await
        .unwrap();
    assert!(link.url.starts_with(api), "{} is served by the API", link.url);
    w.drop().await;
    let w = world_on_s3(false).await;
    let file = w.upload_owned("picture.png", png(4, 4)).await;
    let link = w
        .files
        .signed_url(who(&w.ana), file.id, &w.services)
        .await
        .unwrap();
    assert!(link.url.starts_with(api), "{} is served by the API", link.url);
    w.drop().await;
}
