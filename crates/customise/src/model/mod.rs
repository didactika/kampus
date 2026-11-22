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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of customisation: resolved terms and flags with where they came from, custom fields with
//! their choices and types, and in [`layout`] and [`theme`] layouts, menus and themes.

pub mod layout;
pub mod package;
pub mod rule;
pub mod theme;

use std::str::FromStr;

use kampus_access::ScopeId;
use kampus_common::{Error, FieldError};
use serde::{Deserialize, Serialize};

kampus_common::define_id! {
    /// Identifies a custom field.
    pub struct FieldId;
}

kampus_common::define_id! {
    /// Identifies the value of a custom field for one entity.
    pub struct FieldValueId;
}

/// Where a resolved customisation came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum Origin {
    /// No scope sets it: what its module declares.
    Declared,
    /// Set in this scope, the nearest one or one that locks it.
    Scope {
        /// The scope.
        scope: ScopeId,
        /// Whether that scope locks it for the scopes below.
        locked: bool,
    },
}

/// The word for a term in a scope and a language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Term {
    /// The term.
    pub code: String,
    /// The language of the forms.
    pub language: String,
    /// The word for one.
    pub singular: String,
    /// The word for several.
    pub plural: String,
    /// Where the forms came from.
    pub origin: Origin,
}

/// Whether a feature is on in a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Flag {
    /// The feature.
    pub code: String,
    /// Whether it is on.
    pub enabled: bool,
    /// Where the value came from.
    pub origin: Origin,
}

/// The type of a custom field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Text, at most its length.
    Text,
    /// A whole number.
    Integer,
    /// A date.
    Date,
    /// Yes or no.
    Boolean,
    /// One of its choices.
    Choice,
    /// Any of its choices.
    Choices,
    /// An account of the campus.
    Account,
}

impl FieldType {
    /// The type as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Date => "date",
            Self::Boolean => "boolean",
            Self::Choice => "choice",
            Self::Choices => "choices",
            Self::Account => "account",
        }
    }
}

impl FromStr for FieldType {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        [
            Self::Text,
            Self::Integer,
            Self::Date,
            Self::Boolean,
            Self::Choice,
            Self::Choices,
            Self::Account,
        ]
        .into_iter()
        .find(|t| t.as_str() == text)
        .ok_or_else(|| {
            Error::validation(
                "customise.field_type_invalid",
                vec![FieldError::new("type", "one_of")],
            )
        })
    }
}

/// Who sees the value of a custom field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldVisibility {
    /// Whoever sees the entity.
    Public,
    /// Only who holds `customise.field.view` there.
    Private,
}

/// A choice of a custom field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Choice {
    /// Its code, stable while its label changes.
    pub code: String,
    /// What it says.
    pub label: String,
}

/// A custom field of a kind of entity, defined in a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Field {
    /// The field.
    pub id: FieldId,
    /// The kind of entity it belongs to.
    pub kind: String,
    /// The scope that defines it; it applies there and below.
    pub scope: ScopeId,
    /// Its code, unique for its kind and scope.
    pub code: String,
    /// Its name.
    pub name: String,
    /// Its type.
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Whether every entity has a value.
    pub required: bool,
    /// Who sees its values.
    pub visibility: FieldVisibility,
    /// The longest text, for text fields.
    pub max_length: Option<i32>,
    /// Its choices, in their order, for choice fields.
    pub choices: Vec<Choice>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// A custom field to define.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FieldSpec {
    /// Its code.
    pub code: String,
    /// Its name.
    pub name: String,
    /// Its type.
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Whether every entity has a value.
    #[serde(default)]
    pub required: bool,
    /// Who sees its values.
    #[serde(default = "public")]
    pub visibility: FieldVisibility,
    /// The longest text, for text fields.
    #[serde(default)]
    pub max_length: Option<i32>,
    /// Its choices, for choice fields.
    #[serde(default)]
    pub choices: Vec<Choice>,
}

/// Values are public unless said otherwise.
const fn public() -> FieldVisibility {
    FieldVisibility::Public
}

/// A custom field with its value for one entity.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct FieldEntry {
    /// The field.
    pub field: Field,
    /// Its value: a string, a number, a date, a boolean, a choice code, a list of choice codes, or an
    /// account; null when absent.
    pub value: serde_json::Value,
}
