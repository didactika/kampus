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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Typed identifiers: every entity has its own identifier type wrapping a UUID version 7, so an
//! identifier of one kind cannot be passed where another is expected.

/// Declares a typed identifier wrapping a UUID version 7.
///
/// The type is serialised as the string form of its UUID, parses from it, and is generated ordered by
/// creation time.
///
/// ```
/// kampus_common::define_id! {
///     /// Identifies a course.
///     pub struct CourseId;
/// }
/// let id = CourseId::generate();
/// assert_eq!(id.to_string().parse::<CourseId>().unwrap(), id);
/// ```
#[macro_export]
macro_rules! define_id {
    ($(#[$meta:meta])* $vis:vis struct $name:ident;) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
            $crate::__private::serde::Serialize, $crate::__private::serde::Deserialize,
        )]
        #[serde(crate = "kampus_common::__private::serde", transparent)]
        $vis struct $name($crate::__private::uuid::Uuid);

        impl $name {
            /// Generates a new identifier, ordered by creation time (UUID version 7).
            #[must_use]
            pub fn generate() -> Self {
                Self($crate::__private::uuid::Uuid::now_v7())
            }

            /// Wraps an existing UUID, for example one read from the database.
            #[must_use]
            pub const fn from_uuid(uuid: $crate::__private::uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// The UUID inside, for binding in queries.
            #[must_use]
            pub const fn as_uuid(&self) -> $crate::__private::uuid::Uuid {
                self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl $crate::__private::schemars::JsonSchema for $name {
            fn inline_schema() -> bool {
                true
            }

            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(
                _: &mut $crate::__private::schemars::SchemaGenerator,
            ) -> $crate::__private::schemars::Schema {
                $crate::__private::schemars::json_schema!({ "type": "string", "format": "uuid" })
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::Error;

            fn from_str(text: &str) -> ::std::result::Result<Self, Self::Err> {
                $crate::__private::uuid::Uuid::parse_str(text).map(Self).map_err(|error| {
                    $crate::Error::validation(
                        "common.id_invalid",
                        vec![$crate::FieldError::new(stringify!($name), "format")],
                    )
                    .with_source(error)
                })
            }
        }
    };
}
