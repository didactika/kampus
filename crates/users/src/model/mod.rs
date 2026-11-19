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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of users: profiles, system groups and their memberships.

use kampus_access::ScopeId;
use kampus_identity::AccountId;
use serde::{Deserialize, Serialize};

kampus_common::define_id! {
    /// Identifies a system group.
    pub struct GroupId;
}

/// The person behind an account.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Profile {
    /// The given name.
    pub given_name: Option<String>,
    /// The family name.
    pub family_name: Option<String>,
    /// How the person is shown; the given and family names when absent.
    pub display_name: Option<String>,
    /// The language of the person, a BCP 47 tag; the campus language when absent.
    pub language_code: Option<String>,
    /// The time zone of the person, an IANA name; the campus zone when absent.
    pub time_zone: Option<String>,
    /// The version, for `If-Match`; 0 while the profile was never saved.
    pub version: i32,
}

/// The fields of a profile a person or a manager edits; every field is replaced.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ProfileFields {
    /// The given name.
    pub given_name: Option<String>,
    /// The family name.
    pub family_name: Option<String>,
    /// How the person is shown.
    pub display_name: Option<String>,
    /// A BCP 47 language tag.
    pub language_code: Option<String>,
    /// An IANA time zone name.
    pub time_zone: Option<String>,
}

/// The profile field a rule of a system group compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleField {
    /// The given name.
    GivenName,
    /// The family name.
    FamilyName,
    /// The language.
    LanguageCode,
    /// The time zone.
    TimeZone,
}

impl RuleField {
    /// The column of the field.
    #[must_use]
    pub const fn column(self) -> &'static str {
        match self {
            Self::GivenName => "given_name",
            Self::FamilyName => "family_name",
            Self::LanguageCode => "language_code",
            Self::TimeZone => "time_zone",
        }
    }
}

/// The rule of a system group: the people of its campus whose profile field equals a value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupRule {
    /// The field compared.
    pub field: RuleField,
    /// The value it must equal.
    pub equals: String,
}

/// How the members of a system group are chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum GroupKind {
    /// By hand.
    Manual,
    /// By a rule over profiles, applied when asked.
    Rule(GroupRule),
}

/// Where a membership comes from; a rule or a sync removes only the memberships it made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Added by a person.
    Manual,
    /// Added by the group's rule.
    Rule,
    /// Added by a module that synchronises the group.
    Sync,
}

impl Origin {
    /// The origin as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Rule => "rule",
            Self::Sync => "sync",
        }
    }

    /// Reads a stored origin.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        match text {
            "rule" => Self::Rule,
            "sync" => Self::Sync,
            _ => Self::Manual,
        }
    }
}

/// A system group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SystemGroup {
    /// The group.
    pub id: GroupId,
    /// The campus or category it lives in.
    pub scope: ScopeId,
    /// Its code, unique in its scope.
    pub code: String,
    /// Its name.
    pub name: String,
    /// How its members are chosen.
    pub kind: GroupKind,
}

/// A member of a system group, once per origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Member {
    /// The account.
    pub account: AccountId,
    /// Where the membership comes from.
    pub origin: Origin,
}
