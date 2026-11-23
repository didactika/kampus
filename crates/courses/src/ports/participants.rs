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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Who takes part in a course, which enrolment knows: only participants may be members of its groups, and
//! a published course with participants cannot go back to draft.

use async_trait::async_trait;
use kampus_access::PrincipalId;
use kampus_common::{Result, Timestamp};

use crate::CourseId;

/// Who takes part in a course.
#[async_trait]
pub trait Participants: Send + Sync {
    /// Whether the account takes part in the course.
    async fn takes_part(&self, course: CourseId, account: PrincipalId) -> Result<bool>;

    /// Whether anyone takes part in the course.
    async fn any(&self, course: CourseId) -> Result<bool>;

    /// Whether anyone takes part in the course, read on a connection the caller holds, such as the transaction
    /// of a change; ports that read no database need not override it.
    async fn any_in(&self, _conn: &mut sqlx::PgConnection, course: CourseId) -> Result<bool> {
        self.any(course).await
    }

    /// Everyone who takes part in the course now.
    async fn accounts(&self, course: CourseId) -> Result<Vec<PrincipalId>>;

    /// Since when the account takes part in the course without a break, when it does.
    async fn since(&self, course: CourseId, account: PrincipalId) -> Result<Option<Timestamp>>;

    /// Whether two accounts take part in a course together now.
    async fn together(&self, one: PrincipalId, other: PrincipalId) -> Result<bool>;
}

/// Everyone of the course's campus takes part and nobody is counted, until enrolment answers.
#[derive(Debug, Default, Clone, Copy)]
pub struct CampusMembers;

#[async_trait]
impl Participants for CampusMembers {
    async fn takes_part(&self, _course: CourseId, _account: PrincipalId) -> Result<bool> {
        Ok(true)
    }

    async fn any(&self, _course: CourseId) -> Result<bool> {
        Ok(false)
    }

    async fn accounts(&self, _course: CourseId) -> Result<Vec<PrincipalId>> {
        Ok(Vec::new())
    }

    async fn since(&self, _course: CourseId, _account: PrincipalId) -> Result<Option<Timestamp>> {
        Ok(None)
    }

    async fn together(&self, _one: PrincipalId, _other: PrincipalId) -> Result<bool> {
        Ok(true)
    }
}
