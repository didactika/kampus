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

//! Files through the server, as a client sees them: a profile picture uploaded through the one pipeline,
//! attached by users, read by whoever signed in through a signed link, and served so it never runs.

use axum::http::{Method, StatusCode, header};
use serde_json::json;

use crate::site::{Site, site};

/// A tiny PNG: its signature and the start of a header.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x02\0\0\0";

impl Site {
    /// Uploads a picture through the whole pipeline; returns the file.
    async fn upload_picture(&self, session: &str) -> String {
        let campus = self.campus.to_string();
        let opened = self
            .call(
                Method::POST,
                "/api/v1/files/uploads",
                Some(session),
                &[],
                Some(
                    json!({ "area": "users.picture", "scope": campus, "name": "me.png", "size": PNG.len() }),
                ),
            )
            .await;
        assert_eq!(opened.status, StatusCode::CREATED, "{}", opened.body);
        let upload = opened.body["id"].as_str().unwrap().to_owned();
        let received = self
            .send_bytes(
                Method::PUT,
                &format!("/api/v1/files/uploads/{upload}"),
                session,
                PNG,
            )
            .await;
        assert_eq!(received.status, StatusCode::OK, "{}", received.body);
        let completed = self
            .call(
                Method::POST,
                &format!("/api/v1/files/uploads/{upload}"),
                Some(session),
                &[],
                None,
            )
            .await;
        assert_eq!(completed.status, StatusCode::CREATED, "{}", completed.body);
        assert_eq!(completed.body["media_type"], "image/png");
        completed.body["id"].as_str().unwrap().to_owned()
    }
}

#[tokio::test]
async fn a_profile_picture_is_uploaded_attached_and_served_through_a_signed_link() {
    let site = site().await;
    let admin = site.session("admin").await;
    let file = site.upload_picture(&admin).await;

    let set = site
        .call(
            Method::PUT,
            "/api/v1/users/me/picture",
            Some(&admin),
            &[],
            Some(json!({ "file": file })),
        )
        .await;
    assert_eq!(set.status, StatusCode::NO_CONTENT, "{}", set.body);
    let (_, learner) = site.learner(&admin, "lea@example.org").await;
    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&admin), &[], None)
        .await;
    let admin_account = me.body["id"].as_str().unwrap();
    let picture = site
        .call(
            Method::GET,
            &format!("/api/v1/users/accounts/{admin_account}/picture"),
            Some(&learner),
            &[],
            None,
        )
        .await;
    assert_eq!(picture.body["file"], file.as_str(), "{}", picture.body);
    let link = site
        .call(
            Method::POST,
            &format!("/api/v1/files/files/{file}/url"),
            Some(&learner),
            &[],
            None,
        )
        .await;
    assert_eq!(link.status, StatusCode::OK, "{}", link.body);
    let url = link.body["url"].as_str().unwrap();
    let path = &url[url.find("/api/v1/").unwrap()..];
    let served = site.call(Method::GET, path, None, &[], None).await;
    assert_eq!(served.status, StatusCode::OK);
    assert_eq!(served.headers[header::CONTENT_TYPE], "image/png");
    assert_eq!(served.headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert!(
        served.headers[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .contains("sandbox")
    );
    let tampered = site
        .call(
            Method::GET,
            &path.replace("signature=", "signature=x"),
            None,
            &[],
            None,
        )
        .await;
    assert_eq!(tampered.status, StatusCode::NOT_FOUND);
    let removed = site
        .call(
            Method::DELETE,
            "/api/v1/users/me/picture",
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    site.drop().await;
}
