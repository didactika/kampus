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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The concepts of access control: identifiers, scope kinds, permission values, risks, declarations and
//! the actor of a check.

use std::fmt;
use std::str::FromStr;

use kampus_common::{Error, FieldError};
use serde::Serialize;

kampus_common::define_id! {
    /// Identifies a scope, a node of the scope tree.
    pub struct ScopeId;
}

kampus_common::define_id! {
    /// Identifies a principal: anything that can hold grants (DEC-079).
    pub struct PrincipalId;
}

kampus_common::define_id! {
    /// Identifies a role of a campus.
    pub struct RoleId;
}

kampus_common::define_id! {
    /// Identifies a grant of a role to a principal in a scope.
    pub struct GrantId;
}

kampus_common::define_id! {
    /// Identifies a "view as" session.
    pub struct ViewAsId;
}

/// The kinds of scope, in the order of the tree (RN-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    /// The whole installation.
    Platform,
    /// One tenant of a multi-tenant installation.
    Tenant,
    /// One campus.
    Campus,
    /// A category of courses, possibly nested.
    Category,
    /// A course.
    Course,
    /// A section of a course, possibly nested (DEC-007).
    Section,
    /// An activity or material.
    Activity,
    /// A widget shown in some scope.
    Widget,
    /// A person, hanging from their campus.
    User,
}

impl ScopeKind {
    /// Every kind.
    pub const ALL: [Self; 9] = [
        Self::Platform,
        Self::Tenant,
        Self::Campus,
        Self::Category,
        Self::Course,
        Self::Section,
        Self::Activity,
        Self::Widget,
        Self::User,
    ];

    /// The name of the kind in the database and the API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Platform => "platform",
            Self::Tenant => "tenant",
            Self::Campus => "campus",
            Self::Category => "category",
            Self::Course => "course",
            Self::Section => "section",
            Self::Activity => "activity",
            Self::Widget => "widget",
            Self::User => "user",
        }
    }

    /// How deep the kind sits in the tree; a permission is meaningful only down to its lowest kind (RN-012).
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Platform => 0,
            Self::Tenant => 1,
            Self::Campus => 2,
            Self::Category | Self::User => 3,
            Self::Course => 4,
            Self::Section => 5,
            Self::Activity => 6,
            Self::Widget => 7,
        }
    }

    /// Whether a scope of this kind may hang from a parent of `parent` kind (RN-001).
    #[must_use]
    pub const fn may_hang_from(self, parent: Self) -> bool {
        match self {
            Self::Platform => false,
            Self::Tenant => matches!(parent, Self::Platform),
            Self::Campus => matches!(parent, Self::Platform | Self::Tenant),
            Self::Category => matches!(parent, Self::Campus | Self::Category),
            Self::Course => matches!(parent, Self::Category),
            Self::Section | Self::Activity => matches!(parent, Self::Course | Self::Section),
            Self::User => matches!(parent, Self::Campus),
            Self::Widget => !matches!(parent, Self::Widget),
        }
    }
}

impl FromStr for ScopeKind {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == text)
            .ok_or_else(|| {
                Error::validation(
                    "access.scope_kind_invalid",
                    vec![FieldError::new("kind", "one_of")],
                )
            })
    }
}

impl fmt::Display for ScopeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The value a role gives a permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    /// The role allows it.
    Allow,
    /// The role does not allow it here; a lower scope may allow it again.
    Deny,
    /// No role and no lower scope may allow it (RN-005).
    Forbid,
}

impl Value {
    /// The name of the value in the database and the API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Forbid => "forbid",
        }
    }
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            "forbid" => Ok(Self::Forbid),
            _ => Err(Error::validation(
                "access.value_invalid",
                vec![FieldError::new("value", "one_of")],
            )),
        }
    }
}

/// Whether a permission only reads or also changes something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    /// Reads.
    Read,
    /// Changes something.
    Write,
}

/// What a permission can expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    /// Personal data of other people.
    PersonalData,
    /// The configuration of the campus.
    Configuration,
    /// Content that runs in other people's browsers.
    ScriptInjection,
    /// Losing data.
    DataLoss,
    /// Giving oneself or others more rights.
    TrustEscalation,
    /// Messages to many people.
    Spam,
}

impl Risk {
    /// The name of the risk in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PersonalData => "personal_data",
            Self::Configuration => "configuration",
            Self::ScriptInjection => "script_injection",
            Self::DataLoss => "data_loss",
            Self::TrustEscalation => "trust_escalation",
            Self::Spam => "spam",
        }
    }
}

/// A permission as a module declares it (RN-021, DEC-015).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionDeclaration {
    /// The code, `<module>.<entity>.<action>`.
    pub code: &'static str,
    /// Read or write.
    pub kind: PermissionKind,
    /// What it can expose.
    pub risks: &'static [Risk],
    /// The lowest kind of scope where it means something (RN-012).
    pub lowest_kind: ScopeKind,
}

/// A role template as a module declares it: default values that roles are created from (RN-017).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleTemplateDeclaration {
    /// The code (`manager`, `teacher`…).
    pub code: &'static str,
    /// The version; a new version is a new row, existing roles compare against it.
    pub version: u32,
    /// The values the template gives.
    pub values: &'static [(&'static str, Value)],
}

/// Who a check is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A principal with its grants.
    Principal(PrincipalId),
    /// Someone not signed in, in a campus: only what the campus's guest role reads (RN-009).
    Visitor(ScopeId),
    /// A principal seeing a scope as another role, read-only (RN-018).
    ViewAs(PrincipalId, ViewAsId),
}

/// The kind of an allowance between roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowanceKind {
    /// May grant the target role.
    Grant,
    /// May override the target role.
    Override,
    /// May view as the target role.
    ViewAs,
}

impl AllowanceKind {
    /// The name of the allowance in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Override => "override",
            Self::ViewAs => "view_as",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScopeKind;

    #[test]
    fn kinds_follow_the_tree() {
        assert!(ScopeKind::Section.may_hang_from(ScopeKind::Section));
        assert!(ScopeKind::Campus.may_hang_from(ScopeKind::Tenant));
        assert!(!ScopeKind::Course.may_hang_from(ScopeKind::Campus));
        assert!(!ScopeKind::Platform.may_hang_from(ScopeKind::Platform));
        assert_eq!("section".parse::<ScopeKind>().unwrap(), ScopeKind::Section);
        assert!("room".parse::<ScopeKind>().is_err());
    }
}
