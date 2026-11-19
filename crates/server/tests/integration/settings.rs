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

//! Settings through the server: the identity policy of a campus comes from its settings, so an operator
//! opens sign-up and lengthens passwords without a new release.

use axum::http::{Method, StatusCode};
use kampus_settings::Settings;
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn the_policy_of_a_campus_follows_its_settings() {
    let site = site().await;
    let sign_up = json!({ "email": "lea@example.org", "accepted_terms": true });
    let closed = site
        .call(
            Method::POST,
            "/api/v1/auth/sign-up",
            None,
            &[],
            Some(sign_up.clone()),
        )
        .await;
    assert_eq!(closed.status, StatusCode::FORBIDDEN);
    assert_eq!(closed.code(), "identity.sign_up_closed");

    let settings = site.context.service::<Settings>().unwrap();
    settings
        .set_as_operator("identity.self_sign_up", site.campus, Some(json!(true)), false)
        .await
        .unwrap();
    settings
        .set_as_operator(
            "identity.password_min_length",
            site.campus,
            Some(json!(40)),
            false,
        )
        .await
        .unwrap();
    let open = site
        .call(Method::POST, "/api/v1/auth/sign-up", None, &[], Some(sign_up))
        .await;
    assert_eq!(open.status, StatusCode::ACCEPTED, "{}", open.body);
    let token = site.token_sent_to("lea@example.org");
    let short = site
        .call(
            Method::POST,
            "/api/v1/auth/confirm",
            None,
            &[],
            Some(json!({ "token": token, "password": "only twenty-six characters" })),
        )
        .await;
    assert_eq!(
        short.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a longer password is asked now"
    );
    site.drop().await;
}
