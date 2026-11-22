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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of enrolment: method instances of a course, enrolments with their status and period,
//! and requests to join.

use std::str::FromStr;

use kampus_access::{PrincipalId, RoleId};
use kampus_common::{Error, FieldError, Timestamp};
use kampus_courses::{CourseId, GroupId};
use serde::{Deserialize, Serialize};

kampus_common::define_id! {
    /// Identifies a method instance.
    pub struct InstanceId;
}

kampus_common::define_id! {
    /// Identifies an enrolment.
    pub struct EnrolmentId;
}

kampus_common::define_id! {
    /// Identifies a request to join.
    pub struct RequestId;
}

/// The methods this module provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    /// People enrolled by someone who manages the course.
    Manual,
    /// People who enrol themselves, with a key or an approval when the instance asks.
    #[serde(rename = "self")]
    SelfEnrol,
    /// The members of a system group, kept in step with it.
    SystemGroup,
}

impl Method {
    /// The code as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::SelfEnrol => "self",
            Self::SystemGroup => "system_group",
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "manual" => Ok(Self::Manual),
            "self" => Ok(Self::SelfEnrol),
            "system_group" => Ok(Self::SystemGroup),
            _ => Err(Error::validation(
                "enrol.method_unknown",
                vec![FieldError::new("method", "one_of")],
            )),
        }
    }
}

/// What happens to an enrolment when its period ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryAction {
    /// The enrolment is suspended and kept.
    Suspend,
    /// The enrolment is removed; records and progress stay.
    Remove,
    /// The person keeps taking part, without the role.
    KeepWithoutRole,
}

impl ExpiryAction {
    /// The action as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Suspend => "suspend",
            Self::Remove => "remove",
            Self::KeepWithoutRole => "keep_without_role",
        }
    }

    /// Reads a stored action.
    #[must_use]
    pub fn read(text: &str) -> Self {
        match text {
            "remove" => Self::Remove,
            "keep_without_role" => Self::KeepWithoutRole,
            _ => Self::Suspend,
        }
    }
}

/// The status of an enrolment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Takes part while the instance is enabled and now is within the period.
    Active,
    /// Kept with its data, without taking part.
    Suspended,
}

impl Status {
    /// The status as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
        }
    }

    /// Reads a stored status.
    #[must_use]
    pub fn read(text: &str) -> Self {
        if text == "suspended" {
            Self::Suspended
        } else {
            Self::Active
        }
    }
}

/// A method configured in a course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Instance {
    /// The instance.
    pub id: InstanceId,
    /// Its course.
    pub course: CourseId,
    /// Its method.
    pub method: Method,
    /// Its name, unique in the course.
    pub name: String,
    /// The role its enrolments give.
    pub role: RoleId,
    /// Whether it gives participation.
    pub enabled: bool,
    /// When people may start joining.
    pub open_from: Option<Timestamp>,
    /// When people may no longer join.
    pub open_until: Option<Timestamp>,
    /// How many days an enrolment lasts when it gives no end; open when absent.
    pub default_days: Option<i32>,
    /// How many enrolments it holds at most.
    pub capacity: Option<i32>,
    /// Whether joining waits for an approval.
    pub requires_approval: bool,
    /// What happens when an enrolment ends.
    pub expiry_action: ExpiryAction,
    /// Whether joining needs a key.
    pub has_key: bool,
    /// The system group it follows, for that method.
    pub source: Option<kampus_users::GroupId>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// The fields of an instance its course managers set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct InstanceFields {
    /// Its name.
    pub name: String,
    /// The role its enrolments give.
    pub role: RoleId,
    /// Whether it gives participation.
    #[serde(default = "enabled")]
    pub enabled: bool,
    /// When people may start joining.
    #[serde(default)]
    pub open_from: Option<Timestamp>,
    /// When people may no longer join.
    #[serde(default)]
    pub open_until: Option<Timestamp>,
    /// How many days an enrolment lasts when it gives no end.
    #[serde(default)]
    pub default_days: Option<i32>,
    /// How many enrolments it holds at most.
    #[serde(default)]
    pub capacity: Option<i32>,
    /// Whether joining waits for an approval.
    #[serde(default)]
    pub requires_approval: bool,
    /// What happens when an enrolment ends.
    #[serde(default = "suspend")]
    pub expiry_action: ExpiryAction,
}

/// Instances are enabled unless said otherwise.
const fn enabled() -> bool {
    true
}

/// Enrolments are suspended when they end unless said otherwise.
const fn suspend() -> ExpiryAction {
    ExpiryAction::Suspend
}

/// A person's enrolment through an instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Enrolment {
    /// The enrolment.
    pub id: EnrolmentId,
    /// Its instance.
    pub instance: InstanceId,
    /// Its course.
    pub course: CourseId,
    /// The account.
    pub account: PrincipalId,
    /// Its status.
    pub status: Status,
    /// When it starts.
    pub starts_at: Timestamp,
    /// When it ends; open when absent.
    pub ends_at: Option<Timestamp>,
    /// Who created it, when a person did.
    pub created_by: Option<PrincipalId>,
    /// Why, as told by who made the last change.
    pub reason: Option<String>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// A change to an enrolment.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
pub struct EnrolmentChange {
    /// Its status.
    pub status: Status,
    /// When it starts; unchanged when absent.
    #[serde(default)]
    pub starts_at: Option<Timestamp>,
    /// When it ends; open when absent.
    #[serde(default)]
    pub ends_at: Option<Timestamp>,
    /// Why.
    #[serde(default)]
    pub reason: Option<String>,
}

/// The period of a new enrolment; the instance's default when absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, schemars::JsonSchema)]
pub struct Period {
    /// When it starts; now when absent.
    #[serde(default)]
    pub starts_at: Option<Timestamp>,
    /// When it ends; the instance's default duration, or open, when absent.
    #[serde(default)]
    pub ends_at: Option<Timestamp>,
}

/// The state of a request to join.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestState {
    /// Waiting for a decision.
    Pending,
    /// Approved: the enrolment exists.
    Approved,
    /// Rejected.
    Rejected,
    /// Withdrawn by the person.
    Withdrawn,
}

impl RequestState {
    /// Reads a stored state.
    #[must_use]
    pub fn read(text: &str) -> Self {
        match text {
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            "withdrawn" => Self::Withdrawn,
            _ => Self::Pending,
        }
    }
}

/// A request to join a course through an instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Request {
    /// The request.
    pub id: RequestId,
    /// Its instance.
    pub instance: InstanceId,
    /// The account asking.
    pub account: PrincipalId,
    /// What they wrote.
    pub message: Option<String>,
    /// The group of the key they gave, when any.
    pub group: Option<GroupId>,
    /// Its state.
    pub state: RequestState,
    /// When it was made.
    pub created_at: Timestamp,
}

/// The outcome of enrolling oneself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Joined {
    /// Enrolled.
    Enrolled {
        /// The enrolment.
        enrolment: EnrolmentId,
    },
    /// Waiting for an approval.
    Requested {
        /// The request.
        request: RequestId,
    },
}

/// What a sync of an instance with its source did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, schemars::JsonSchema)]
pub struct Synced {
    /// Enrolments added.
    pub added: usize,
    /// Enrolments removed.
    pub removed: usize,
}

/// What an expiry run did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Expired {
    /// Enrolments suspended.
    pub suspended: usize,
    /// Enrolments removed.
    pub removed: usize,
    /// Enrolments that kept participation without the role.
    pub kept_without_role: usize,
}
