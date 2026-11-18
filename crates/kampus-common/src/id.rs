// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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

//! Typed identifiers (DEC-021): every entity has its own identifier type wrapping a UUID version 7, so an
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

#[cfg(test)]
#[allow(
    unreachable_pub,
    dead_code,
    reason = "the identifier of the tests is used only partly"
)]
mod tests {
    crate::define_id! {
        /// An identifier used only by these tests.
        struct SampleId;
    }

    #[test]
    fn generated_identifiers_are_version_7_and_ordered() {
        let first = SampleId::generate();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = SampleId::generate();
        assert_eq!(first.as_uuid().get_version_num(), 7);
        assert!(first < second);
    }

    #[test]
    fn identifiers_round_trip_as_text_and_json() {
        let id = SampleId::generate();
        assert_eq!(id.to_string().parse::<SampleId>().unwrap(), id);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{id}\""));
        assert_eq!(serde_json::from_str::<SampleId>(&json).unwrap(), id);
    }

    #[test]
    fn invalid_text_is_a_validation_error() {
        let error = "42".parse::<SampleId>().unwrap_err();
        assert_eq!(error.code(), "common.id_invalid");
        assert_eq!(error.fields()[0].field, "SampleId");
    }
}
