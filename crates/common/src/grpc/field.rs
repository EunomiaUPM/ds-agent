/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Parsing of proto string fields into domain types, failing with `INVALID_ARGUMENT` named by field.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use tonic::Status;
use urn::Urn;

/// Builder for field-scoped `INVALID_ARGUMENT` statuses (`"<field>: <reason>"`).
pub struct InvalidField;

impl InvalidField {
    pub fn status(field: &str, reason: impl std::fmt::Display) -> Status {
        Status::invalid_argument(format!("{field}: {reason}"))
    }
}

/// Extension trait parsing a scalar proto string field (proto3 uses `""` as absent).
pub trait ProtoField {
    /// `Some(s)` if the field is non-empty, `None` if it is `""`.
    fn non_empty(&self) -> Option<&str>;
    /// Parses the field with `FromStr`; empty is an error.
    fn parsed<T>(&self, field: &str) -> Result<T, Status>
    where
        T: FromStr,
        T::Err: std::fmt::Display;
    /// Parses the field with `FromStr`; empty is `None`.
    fn opt_parsed<T>(&self, field: &str) -> Result<Option<T>, Status>
    where
        T: FromStr,
        T::Err: std::fmt::Display;
    fn urn(&self, field: &str) -> Result<Urn, Status>;
    fn opt_urn(&self, field: &str) -> Result<Option<Urn>, Status>;
    fn rfc3339(&self, field: &str) -> Result<DateTime<Utc>, Status>;
    fn opt_rfc3339(&self, field: &str) -> Result<Option<DateTime<Utc>>, Status>;
    fn json(&self, field: &str) -> Result<serde_json::Value, Status>;
    fn opt_json(&self, field: &str) -> Result<Option<serde_json::Value>, Status>;
}

impl ProtoField for str {
    fn non_empty(&self) -> Option<&str> {
        (!self.is_empty()).then_some(self)
    }

    fn parsed<T>(&self, field: &str) -> Result<T, Status>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        if self.is_empty() {
            return Err(InvalidField::status(field, "is required"));
        }
        self.parse::<T>()
            .map_err(|e| InvalidField::status(field, e))
    }

    fn opt_parsed<T>(&self, field: &str) -> Result<Option<T>, Status>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.non_empty().map(|s| s.parsed(field)).transpose()
    }

    fn urn(&self, field: &str) -> Result<Urn, Status> {
        if self.is_empty() {
            return Err(InvalidField::status(field, "is required"));
        }
        Urn::from_str(self).map_err(|e| InvalidField::status(field, format!("invalid URN — {e}")))
    }

    fn opt_urn(&self, field: &str) -> Result<Option<Urn>, Status> {
        self.non_empty().map(|s| s.urn(field)).transpose()
    }

    fn rfc3339(&self, field: &str) -> Result<DateTime<Utc>, Status> {
        if self.is_empty() {
            return Err(InvalidField::status(field, "is required"));
        }
        DateTime::parse_from_rfc3339(self)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| InvalidField::status(field, format!("invalid RFC3339 — {e}")))
    }

    fn opt_rfc3339(&self, field: &str) -> Result<Option<DateTime<Utc>>, Status> {
        self.non_empty().map(|s| s.rfc3339(field)).transpose()
    }

    fn json(&self, field: &str) -> Result<serde_json::Value, Status> {
        if self.is_empty() {
            return Err(InvalidField::status(field, "is required"));
        }
        serde_json::from_str(self)
            .map_err(|e| InvalidField::status(field, format!("invalid JSON — {e}")))
    }

    fn opt_json(&self, field: &str) -> Result<Option<serde_json::Value>, Status> {
        self.non_empty().map(|s| s.json(field)).transpose()
    }
}

/// Extension trait decoding a proto enum carried as `i32`.
pub trait ProtoEnum {
    /// Decodes the wire integer into a prost enum; unknown values name the field.
    fn proto_enum<E: TryFrom<i32>>(self, field: &str) -> Result<E, Status>;
}

impl ProtoEnum for i32 {
    fn proto_enum<E: TryFrom<i32>>(self, field: &str) -> Result<E, Status> {
        E::try_from(self)
            .map_err(|_| InvalidField::status(field, format!("unknown enum value {self}")))
    }
}

/// Extension trait parsing a `repeated string` proto field.
pub trait ProtoFieldList {
    /// Parses every element as a URN; the first failure names the field and its index.
    fn urns(&self, field: &str) -> Result<Vec<Urn>, Status>;
}

impl ProtoFieldList for [String] {
    fn urns(&self, field: &str) -> Result<Vec<Urn>, Status> {
        self.iter()
            .enumerate()
            .map(|(i, s)| s.urn(&format!("{field}[{i}]")))
            .collect()
    }
}
