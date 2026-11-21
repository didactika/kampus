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

//! Deletions: asked by the person, approved by someone who may, erased by every module, left as a tombstone,
//! and kept waiting by a legal hold.

use kampus_common::ErrorKind;
use kampus_identity::{AccountId, AccountStatus};
use kampus_module::ServiceContext;

use crate::world::{World, who, world};

/// The services the handlers and the tombstone need.
fn services(w: &World) -> ServiceContext {
    ServiceContext::default()
        .with(w.records.clone())
        .with(w.progress.clone())
        .with(w.privacy.clone())
        .with(w.identity.clone())
}

/// The status of the account of an actor, read by the administrator.
async fn status(w: &World, actor: &kampus_access::Actor) -> AccountStatus {
    w.identity
        .account(&w.admin, AccountId::from_uuid(who(actor).as_uuid()))
        .await
        .unwrap()
        .status
}

#[tokio::test]
async fn a_deletion_waits_for_approval_and_then_leaves_a_tombstone() {
    let w = world().await;
    let request = w
        .privacy
        .request_deletion(&w.lea, who(&w.lea), "I am leaving")
        .await
        .unwrap();
    assert_eq!(request.state, "awaiting_approval");
    assert_eq!(
        w.privacy
            .request_deletion(&w.lea, who(&w.lea), "again")
            .await
            .unwrap_err()
            .code(),
        "privacy.deletion_open"
    );
    assert_eq!(
        w.privacy.run(&services(&w)).await.unwrap(),
        0,
        "nothing runs before the approval"
    );
    assert_eq!(w.privacy.awaiting_approval(&w.admin).await.unwrap().len(), 1);
    assert_eq!(
        w.privacy.awaiting_approval(&w.max).await.unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
    assert_eq!(
        w.privacy
            .decide(&w.max, request.id, true)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );

    let approved = w.privacy.decide(&w.admin, request.id, true).await.unwrap();
    assert_eq!(approved.state, "pending");
    assert_eq!(approved.approved_by, Some(who(&w.admin)));
    assert_eq!(
        w.privacy
            .decide(&w.admin, request.id, true)
            .await
            .unwrap_err()
            .code(),
        "privacy.request_not_waiting"
    );
    w.privacy.run(&services(&w)).await.unwrap();
    let done = w.privacy.request(&w.lea, request.id).await.unwrap();
    assert_eq!(done.state, "completed");
    assert_eq!(status(&w, &w.lea).await, AccountStatus::Deleted);
    assert_eq!(status(&w, &w.max).await, AccountStatus::Active);
    assert_eq!(
        w.privacy.package(&w.lea, request.id).await.unwrap_err().code(),
        "privacy.no_package"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_rejected_deletion_changes_nothing() {
    let w = world().await;
    let request = w
        .privacy
        .request_deletion(&w.lea, who(&w.lea), "I am leaving")
        .await
        .unwrap();
    let rejected = w.privacy.decide(&w.admin, request.id, false).await.unwrap();
    assert_eq!(rejected.state, "rejected");
    w.privacy.run(&services(&w)).await.unwrap();
    assert_eq!(status(&w, &w.lea).await, AccountStatus::Active);
    let again = w
        .privacy
        .request_deletion(&w.lea, who(&w.lea), "I changed my mind again")
        .await
        .unwrap();
    assert_eq!(
        again.state, "awaiting_approval",
        "a rejected one does not block a new one"
    );
    w.drop().await;
}

#[tokio::test]
async fn whoever_may_approve_does_not_wait_and_a_reason_is_required() {
    let w = world().await;
    assert_eq!(
        w.privacy
            .request_deletion(&w.admin, who(&w.max), " ")
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Validation
    );
    assert_eq!(
        w.privacy
            .request_deletion(&w.lea, who(&w.max), "not mine")
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    let request = w
        .privacy
        .request_deletion(&w.admin, who(&w.max), "requested by the person in writing")
        .await
        .unwrap();
    assert_eq!(request.state, "pending");
    assert_eq!(request.approved_by, Some(who(&w.admin)));
    w.drop().await;
}

#[tokio::test]
async fn a_legal_hold_keeps_the_deletion_waiting_until_it_is_released() {
    let w = world().await;
    let hold = w
        .privacy
        .place_hold(&w.admin, Some(who(&w.max)), "pending investigation", None)
        .await
        .unwrap();
    assert_eq!(
        w.privacy
            .place_hold(&w.lea, None, "no right", None)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    let request = w
        .privacy
        .request_deletion(&w.admin, who(&w.max), "requested by the person in writing")
        .await
        .unwrap();
    w.privacy.run(&services(&w)).await.unwrap();
    assert_eq!(
        w.privacy.request(&w.admin, request.id).await.unwrap().state,
        "held"
    );
    assert_eq!(status(&w, &w.max).await, AccountStatus::Active);
    assert_eq!(w.privacy.holds(&w.admin).await.unwrap().len(), 1);

    let released = w.privacy.release_hold(&w.admin, hold.id).await.unwrap();
    assert!(released.released_at.is_some());
    w.privacy.run(&services(&w)).await.unwrap();
    assert_eq!(
        w.privacy.request(&w.admin, request.id).await.unwrap().state,
        "completed"
    );
    assert_eq!(status(&w, &w.max).await, AccountStatus::Deleted);
    w.drop().await;
}

#[tokio::test]
async fn a_hold_on_the_campus_stops_every_deletion() {
    let w = world().await;
    w.privacy
        .place_hold(&w.admin, None, "audit of the campus", None)
        .await
        .unwrap();
    let request = w
        .privacy
        .request_deletion(&w.admin, who(&w.lea), "requested by the person in writing")
        .await
        .unwrap();
    w.privacy.run(&services(&w)).await.unwrap();
    assert_eq!(
        w.privacy.request(&w.admin, request.id).await.unwrap().state,
        "held"
    );
    w.drop().await;
}
