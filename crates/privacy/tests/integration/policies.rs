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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Policies: a campus publishes versions, each person accepts the version in force, and what they accepted is
//! part of their exported data.

use kampus_common::ErrorKind;
use kampus_module::ServiceContext;

use crate::world::{who, world};

#[tokio::test]
async fn a_campus_publishes_versions_and_people_accept_the_latest() {
    let w = world().await;
    let first = w
        .privacy
        .publish_policy(&w.admin, "Terms", "Terms of use", "Be kind.")
        .await
        .unwrap();
    assert_eq!((first.code.as_str(), first.version), ("terms", 1));
    assert_eq!(w.privacy.pending_policies(&w.lea).await.unwrap().len(), 1);

    let accepted = w.privacy.accept_policy(&w.lea, "terms", 1).await.unwrap();
    assert!(accepted.accepted_at.is_some());
    let again = w.privacy.accept_policy(&w.lea, "terms", 1).await.unwrap();
    assert_eq!(
        again.accepted_at, accepted.accepted_at,
        "accepting again changes nothing"
    );
    assert_eq!(w.privacy.pending_policies(&w.lea).await.unwrap().len(), 0);
    assert_eq!(w.privacy.pending_policies(&w.max).await.unwrap().len(), 1);

    let second = w
        .privacy
        .publish_policy(&w.admin, "terms", "Terms of use", "Be kind and brief.")
        .await
        .unwrap();
    assert_eq!(second.version, 2);
    assert_eq!(
        w.privacy.pending_policies(&w.lea).await.unwrap().len(),
        1,
        "a new version has to be accepted again"
    );
    assert_eq!(
        w.privacy
            .accept_policy(&w.lea, "terms", 1)
            .await
            .unwrap_err()
            .code(),
        "privacy.policy_outdated"
    );
    assert_eq!(
        w.privacy
            .accept_policy(&w.lea, "terms", 3)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    w.drop().await;
}

#[tokio::test]
async fn only_who_administers_publishes_and_the_text_is_required() {
    let w = world().await;
    let denied = w
        .privacy
        .publish_policy(&w.lea, "terms", "Terms", "Be kind.")
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let empty = w
        .privacy
        .publish_policy(&w.admin, "terms", "Terms", "  ")
        .await
        .unwrap_err();
    assert_eq!(empty.kind(), ErrorKind::Validation);
    w.drop().await;
}

#[tokio::test]
async fn what_a_person_accepted_is_part_of_their_export() {
    let w = world().await;
    w.privacy
        .publish_policy(&w.admin, "terms", "Terms", "Be kind.")
        .await
        .unwrap();
    w.privacy.accept_policy(&w.lea, "terms", 1).await.unwrap();
    let request = w.privacy.request_export(&w.lea, who(&w.lea)).await.unwrap();
    let services = ServiceContext::default()
        .with(w.records.clone())
        .with(w.progress.clone())
        .with(w.privacy.clone());
    w.privacy.run(&services).await.unwrap();
    let package = w.privacy.package(&w.lea, request.id).await.unwrap();
    let accepted = package["data"]["privacy.acceptances"]["acceptance"]
        .as_array()
        .unwrap();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0]["policy"], "terms");
    w.drop().await;
}
