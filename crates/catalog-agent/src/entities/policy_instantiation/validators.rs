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

//! Checks a parameter value against its type and restrictions.

use std::sync::LazyLock;

use chrono::{DateTime, NaiveDate, NaiveTime};
use regex::Regex;
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::policy_templates::types::{
    ParameterDataType, SelectionAllowedValues, ValidationRestrictions,
};
use crate::entities::policy_templates::ParameterValue;

/// xsd:duration; the "at least one part" rule is checked apart, the regex crate has no lookahead.
static DURATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^-?P(\d+Y)?(\d+M)?(\d+W)?(\d+D)?(T(\d+H)?(\d+M)?(\d+(\.\d+)?S)?)?$")
        .expect("valid duration regex")
});
/// A scheme or prefix, a colon and no whitespace: `https://…`, `urn:…`, `gx:LegalParticipant`.
static IRI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9+.\-]*:\S+$").expect("valid IRI regex"));

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
    #[error("String length {length} is less than minimum {min}")]
    MinLength { length: usize, min: usize },
    #[error("String length {length} is greater than maximum {max}")]
    MaxLength { length: usize, max: usize },
    #[error("Value '{value}' does not match regex pattern")]
    RegexMismatch { value: String },
    #[error("Numeric value {value} is less than minimum {min}")]
    MinValue { value: f64, min: f64 },
    #[error("Numeric value {value} is greater than maximum {max}")]
    MaxValue { value: f64, max: f64 },
    #[error("Value '{value}' is not in the allowed values list")]
    InvalidSelection { value: String },
    #[error("Value '{value}' is not a valid {kind}")]
    InvalidFormat { value: String, kind: &'static str },
    #[error("Value {value} is before minimum allowed {min}")]
    MinDate { value: String, min: String },
    #[error("Value {value} is after maximum allowed {max}")]
    MaxDate { value: String, max: String },
    #[error("List has {count} items, fewer than {min}")]
    MinItems { count: usize, min: usize },
    #[error("List has {count} items, more than {max}")]
    MaxItems { count: usize, max: usize },
    #[error("Invalid restriction in template: {0}")]
    RestrictionConfig(String),
}

impl RepoIntoErrors for ValidationError {}

/// Checks one value of a parameter type.
pub trait ParameterValidator: Send + Sync {
    fn validate(&self, value: &ParameterValue, restrictions: &ValidationRestrictions) -> Outcome<()>;
}

/// Strings with length and regex limits; also IRIs, which must look like one.
pub struct StringValidator {
    iri: bool,
}

impl StringValidator {
    fn text<'a>(value: &'a ParameterValue, expected: &str) -> Outcome<&'a str> {
        match value {
            ParameterValue::Text(text) => Ok(text),
            other => Err(ValidationError::TypeMismatch {
                expected: expected.into(),
                got: format!("{other:?}"),
            }
            .into_errors()),
        }
    }
}

impl ParameterValidator for StringValidator {
    fn validate(&self, value: &ParameterValue, restrictions: &ValidationRestrictions) -> Outcome<()> {
        let text = Self::text(value, if self.iri { "IRI" } else { "String" })?;
        if self.iri && !IRI.is_match(text) {
            return Err(ValidationError::InvalidFormat { value: text.into(), kind: "IRI" }.into_errors());
        }
        let length = text.chars().count();
        if let Some(min) = restrictions.min_length.filter(|min| length < *min) {
            return Err(ValidationError::MinLength { length, min }.into_errors());
        }
        if let Some(max) = restrictions.max_length.filter(|max| length > *max) {
            return Err(ValidationError::MaxLength { length, max }.into_errors());
        }
        if let Some(pattern) = &restrictions.regex {
            let re = Regex::new(pattern)
                .map_err(|e| ValidationError::RestrictionConfig(e.to_string()).into_errors())?;
            if !re.is_match(text) {
                return Err(ValidationError::RegexMismatch { value: text.into() }.into_errors());
            }
        }
        Ok(())
    }
}

/// Numbers within a range; integers must have no fractional part.
pub struct NumericValidator {
    integer: bool,
}

impl ParameterValidator for NumericValidator {
    fn validate(&self, value: &ParameterValue, restrictions: &ValidationRestrictions) -> Outcome<()> {
        let expected = if self.integer { "Integer" } else { "Number" };
        let number = match value {
            ParameterValue::Number(n) if !self.integer || n.is_i64() || n.is_u64() => n,
            other => {
                return Err(ValidationError::TypeMismatch {
                    expected: expected.into(),
                    got: format!("{other:?}"),
                }
                .into_errors())
            }
        };
        let value = number.as_f64().unwrap_or_default();
        if let Some(min) = restrictions.min_value.filter(|min| value < *min) {
            return Err(ValidationError::MinValue { value, min }.into_errors());
        }
        if let Some(max) = restrictions.max_value.filter(|max| value > *max) {
            return Err(ValidationError::MaxValue { value, max }.into_errors());
        }
        Ok(())
    }
}

pub struct BooleanValidator;

impl ParameterValidator for BooleanValidator {
    fn validate(&self, value: &ParameterValue, _: &ValidationRestrictions) -> Outcome<()> {
        match value {
            ParameterValue::Boolean(_) => Ok(()),
            other => Err(ValidationError::TypeMismatch {
                expected: "Boolean".into(),
                got: format!("{other:?}"),
            }
            .into_errors()),
        }
    }
}

/// One of the allowed values; a number is compared as its text.
pub struct SelectionValidator;

impl SelectionValidator {
    fn allowed(allowed: &SelectionAllowedValues, value: &str) -> bool {
        match allowed {
            SelectionAllowedValues::Simple(values) => values.iter().any(|v| v == value),
            SelectionAllowedValues::Complex(options) => options.iter().any(|o| o.value == value),
        }
    }
}

impl ParameterValidator for SelectionValidator {
    fn validate(&self, value: &ParameterValue, restrictions: &ValidationRestrictions) -> Outcome<()> {
        let text = match value {
            ParameterValue::Text(text) => text.clone(),
            ParameterValue::Number(n) => n.to_string(),
            other => {
                return Err(ValidationError::TypeMismatch {
                    expected: "Selection".into(),
                    got: format!("{other:?}"),
                }
                .into_errors())
            }
        };
        match &restrictions.values {
            Some(allowed) if !Self::allowed(allowed, &text) => {
                Err(ValidationError::InvalidSelection { value: text }.into_errors())
            }
            _ => Ok(()),
        }
    }
}

/// Lists of strings with an item count and, optionally, allowed values for every item.
pub struct ListValidator;

impl ParameterValidator for ListValidator {
    fn validate(&self, value: &ParameterValue, restrictions: &ValidationRestrictions) -> Outcome<()> {
        let items = match value {
            ParameterValue::List(items) => items,
            other => {
                return Err(ValidationError::TypeMismatch {
                    expected: "List".into(),
                    got: format!("{other:?}"),
                }
                .into_errors())
            }
        };
        let count = items.len();
        if let Some(min) = restrictions.min_items.filter(|min| count < *min) {
            return Err(ValidationError::MinItems { count, min }.into_errors());
        }
        if let Some(max) = restrictions.max_items.filter(|max| count > *max) {
            return Err(ValidationError::MaxItems { count, max }.into_errors());
        }
        if let Some(allowed) = &restrictions.values {
            if let Some(item) = items.iter().find(|item| !SelectionValidator::allowed(allowed, item)) {
                return Err(ValidationError::InvalidSelection { value: item.clone() }.into_errors());
            }
        }
        Ok(())
    }
}

/// Which temporal format a value and its `minDate`/`maxDate` follow.
#[derive(Clone, Copy)]
pub enum Temporal {
    Date,
    Time,
    DateTime,
}

/// Dates, times and date-times in their own format, within `minDate` and `maxDate`.
pub struct TemporalValidator {
    kind: Temporal,
}

impl TemporalValidator {
    /// Seconds from a fixed origin, so the three formats compare the same way.
    fn instant(&self, text: &str) -> Option<i64> {
        match self.kind {
            Temporal::Date => NaiveDate::parse_from_str(text, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|d| d.and_utc().timestamp()),
            Temporal::Time => NaiveTime::parse_from_str(text, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(text, "%H:%M"))
                .ok()
                .map(|t| t.signed_duration_since(NaiveTime::MIN).num_seconds()),
            Temporal::DateTime => DateTime::parse_from_rfc3339(text).ok().map(|d| d.timestamp()),
        }
    }

    fn label(&self) -> &'static str {
        match self.kind {
            Temporal::Date => "date (YYYY-MM-DD)",
            Temporal::Time => "time (HH:MM[:SS])",
            Temporal::DateTime => "RFC 3339 date-time",
        }
    }

    fn bound(&self, text: &str) -> Outcome<i64> {
        self.instant(text).ok_or_else(|| {
            ValidationError::RestrictionConfig(format!("'{text}' is not a {}", self.label())).into_errors()
        })
    }
}

impl ParameterValidator for TemporalValidator {
    fn validate(&self, value: &ParameterValue, restrictions: &ValidationRestrictions) -> Outcome<()> {
        let text = StringValidator::text(value, self.label())?;
        let instant = self
            .instant(text)
            .ok_or_else(|| ValidationError::InvalidFormat { value: text.into(), kind: self.label() }.into_errors())?;
        if let Some(min) = &restrictions.min_date {
            if instant < self.bound(min)? {
                return Err(ValidationError::MinDate { value: text.into(), min: min.clone() }.into_errors());
            }
        }
        if let Some(max) = &restrictions.max_date {
            if instant > self.bound(max)? {
                return Err(ValidationError::MaxDate { value: text.into(), max: max.clone() }.into_errors());
            }
        }
        Ok(())
    }
}

/// xsd:durations: `P` and at least one part, with a `T` only before hours, minutes or seconds.
pub struct DurationValidator;

impl DurationValidator {
    fn is_duration(text: &str) -> bool {
        let body = text.trim_start_matches('-');
        DURATION.is_match(text) && body != "P" && !body.ends_with('T')
    }
}

impl ParameterValidator for DurationValidator {
    fn validate(&self, value: &ParameterValue, _: &ValidationRestrictions) -> Outcome<()> {
        let text = StringValidator::text(value, "Duration")?;
        if Self::is_duration(text) {
            Ok(())
        } else {
            Err(ValidationError::InvalidFormat { value: text.into(), kind: "xsd:duration" }.into_errors())
        }
    }
}

pub struct ValidatorFactory;

impl ValidatorFactory {
    pub fn get_validator(data_type: ParameterDataType) -> Box<dyn ParameterValidator> {
        match data_type {
            ParameterDataType::String => Box::new(StringValidator { iri: false }),
            ParameterDataType::Iri => Box::new(StringValidator { iri: true }),
            ParameterDataType::Integer => Box::new(NumericValidator { integer: true }),
            ParameterDataType::Float => Box::new(NumericValidator { integer: false }),
            ParameterDataType::Boolean => Box::new(BooleanValidator),
            ParameterDataType::Selection => Box::new(SelectionValidator),
            ParameterDataType::List => Box::new(ListValidator),
            ParameterDataType::Duration => Box::new(DurationValidator),
            ParameterDataType::Date => Box::new(TemporalValidator { kind: Temporal::Date }),
            ParameterDataType::Time => Box::new(TemporalValidator { kind: Temporal::Time }),
            ParameterDataType::DateTime => Box::new(TemporalValidator { kind: Temporal::DateTime }),
        }
    }
}
