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

//! The error model shared by every module (DEC-018): a closed set of kinds, a stable code of the module
//! that raised the error, translatable parameters and, for validation, every failing field at once.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The result of a Kampus operation.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The kind of an error, which decides how every client handles it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The object does not exist, or the caller may not know it exists.
    NotFound,
    /// The caller may not do this.
    PermissionDenied,
    /// The request is invalid; the error lists every failing field.
    Validation,
    /// The request conflicts with the current state.
    Conflict,
    /// The resource changed since the caller read it (`If-Match`).
    PreconditionFailed,
    /// The caller sent too many requests.
    RateLimited,
    /// A dependency is not available; retrying later may work.
    Unavailable,
    /// An unexpected failure; only the request id reaches the caller.
    Internal,
}

impl ErrorKind {
    /// The HTTP status this kind maps to in problem details (RFC 9457).
    #[must_use]
    pub const fn http_status(self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::PermissionDenied => 403,
            Self::Validation => 422,
            Self::Conflict => 409,
            Self::PreconditionFailed => 412,
            Self::RateLimited => 429,
            Self::Unavailable => 503,
            Self::Internal => 500,
        }
    }

    /// The name of this kind in problem details and logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::PermissionDenied => "permission_denied",
            Self::Validation => "validation",
            Self::Conflict => "conflict",
            Self::PreconditionFailed => "precondition_failed",
            Self::RateLimited => "rate_limited",
            Self::Unavailable => "unavailable",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failing field of a validation error: which field, and which rule it breaks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    /// The field, as the caller named it (`limit`, `profile.time_zone`).
    pub field: String,
    /// The rule it breaks (`required`, `range`, `format`).
    pub rule: String,
}

impl FieldError {
    /// Creates a field error.
    pub fn new(field: impl Into<String>, rule: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            rule: rule.into(),
        }
    }
}

/// An error of a Kampus operation.
#[derive(Debug)]
pub struct Error {
    /// The kind, which decides how clients handle the error.
    kind: ErrorKind,
    /// The stable code, `<module>.<reason>`.
    code: String,
    /// The parameters of the translated message.
    params: Vec<(String, String)>,
    /// The failing fields of a validation error.
    fields: Vec<FieldError>,
    /// The underlying cause, kept for logs and never shown to callers.
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    /// Creates an error of a kind with its stable code (`<module>.<reason>`).
    pub fn new(kind: ErrorKind, code: impl Into<String>) -> Self {
        Self {
            kind,
            code: code.into(),
            params: Vec::new(),
            fields: Vec::new(),
            source: None,
        }
    }

    /// Creates a `not_found` error.
    pub fn not_found(code: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, code)
    }

    /// Creates a `permission_denied` error.
    pub fn permission_denied(code: impl Into<String>) -> Self {
        Self::new(ErrorKind::PermissionDenied, code)
    }

    /// Creates a `validation` error with every failing field.
    pub fn validation(code: impl Into<String>, fields: Vec<FieldError>) -> Self {
        let mut error = Self::new(ErrorKind::Validation, code);
        error.fields = fields;
        error
    }

    /// Creates a `conflict` error.
    pub fn conflict(code: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, code)
    }

    /// Creates an `unavailable` error.
    pub fn unavailable(code: impl Into<String>) -> Self {
        Self::new(ErrorKind::Unavailable, code)
    }

    /// Creates an `internal` error.
    pub fn internal(code: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, code)
    }

    /// Adds a parameter of the translated message.
    #[must_use]
    pub fn with_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.push((name.into(), value.into()));
        self
    }

    /// Keeps the underlying cause for logs.
    #[must_use]
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// The kind of the error.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The stable code of the error.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The parameters of the translated message.
    #[must_use]
    pub fn params(&self) -> &[(String, String)] {
        &self.params
    }

    /// The failing fields of a validation error.
    #[must_use]
    pub fn fields(&self) -> &[FieldError] {
        &self.fields
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.code)?;
        for (name, value) in &self.params {
            write!(f, " {name}={value}")?;
        }
        for field in &self.fields {
            write!(f, " [{}: {}]", field.field, field.rule)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, ErrorKind, FieldError};

    #[test]
    fn every_kind_maps_to_its_status() {
        let expected = [
            (ErrorKind::NotFound, 404),
            (ErrorKind::PermissionDenied, 403),
            (ErrorKind::Validation, 422),
            (ErrorKind::Conflict, 409),
            (ErrorKind::PreconditionFailed, 412),
            (ErrorKind::RateLimited, 429),
            (ErrorKind::Unavailable, 503),
            (ErrorKind::Internal, 500),
        ];
        for (kind, status) in expected {
            assert_eq!(kind.http_status(), status, "{kind}");
        }
    }

    #[test]
    fn validation_keeps_every_field_and_displays_them() {
        let error = Error::validation(
            "common.page_invalid",
            vec![
                FieldError::new("limit", "range"),
                FieldError::new("after", "cursor"),
            ],
        )
        .with_param("max", "200");
        assert_eq!(error.kind(), ErrorKind::Validation);
        assert_eq!(error.fields().len(), 2);
        assert_eq!(
            error.to_string(),
            "validation: common.page_invalid max=200 [limit: range] [after: cursor]"
        );
    }

    #[test]
    fn the_cause_is_kept_as_source() {
        let error = Error::internal("db.query").with_source(std::io::Error::other("disk"));
        assert_eq!(
            std::error::Error::source(&error)
                .map(ToString::to_string)
                .as_deref(),
            Some("disk")
        );
    }

    #[test]
    fn kinds_serialise_in_snake_case() {
        assert_eq!(
            serde_json::to_string(&ErrorKind::PreconditionFailed).unwrap(),
            "\"precondition_failed\""
        );
    }
}
