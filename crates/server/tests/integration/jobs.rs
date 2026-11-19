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

//! Background work through the server: the jobs every module declares run with the services the server
//! composed, and their schedules are in place.

use kampus_server::wiring::registry::MODULES;
use kampus_tasks::{JobState, Tasks, Worker};
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn the_jobs_of_every_module_run_with_the_services_of_the_server() {
    let site = site().await;
    let tasks = Tasks::new(site.db.clone());
    let worker = Worker::new(
        tasks.clone(),
        MODULES.iter().flat_map(|m| m.jobs.iter().copied()),
        site.context.clone(),
        "test".to_owned(),
    );
    let declared: Vec<&str> = MODULES
        .iter()
        .flat_map(|m| m.jobs.iter().map(|j| j.code))
        .collect();
    for code in &declared {
        tasks.enqueue(code, &json!({}), None).await.unwrap();
    }
    while worker.run_once().await.unwrap() {}
    let failed = tasks.jobs(None, Some(JobState::Failed), 50).await.unwrap();
    assert!(failed.is_empty(), "{failed:?}");
    let done = tasks.jobs(None, Some(JobState::Succeeded), 50).await.unwrap();
    assert_eq!(done.len(), declared.len());
    let scheduled: Vec<String> = tasks
        .schedules()
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.type_code)
        .collect();
    for code in [
        "identity.purge",
        "enrol.sync",
        "courses.run_deletions",
        "tasks.purge",
    ] {
        assert!(scheduled.iter().any(|s| s == code), "{code} is scheduled");
    }
    site.drop().await;
}
