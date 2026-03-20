use crate::entities::common::PolicyTemplateAllowedDefaultValues;
use crate::entities::policy_templates::types::{
    ParameterDataType, SelectionAllowedValues, ValidationRestrictions,
};
use crate::entities::policy_templates::validator::PolicyTemplateError;
use regex::Regex;
use thiserror::Error;
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

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
    #[error("Invalid date format: {0}")]
    DateFormatError(#[from] chrono::ParseError),
    #[error("Date {value} is before minimum allowed date {min}")]
    MinDate { value: String, min: String },
    #[error("Date {value} is after maximum allowed date {max}")]
    MaxDate { value: String, max: String },
    #[error("Invalid regex configuration in template: {0}")]
    RegexConfigError(#[from] regex::Error),
    #[error("Invalid date format in template restriction: {0}")]
    DateConfigError(String),
}

impl RepoIntoErrors for ValidationError {}

pub trait ParameterValidator: Send + Sync {
    fn validate(
        &self,
        value: &PolicyTemplateAllowedDefaultValues,
        restrictions: &ValidationRestrictions,
    ) -> Outcome<()>;
}

pub struct StringValidator;

impl ParameterValidator for StringValidator {
    fn validate(
        &self,
        value: &PolicyTemplateAllowedDefaultValues,
        restrictions: &ValidationRestrictions,
    ) -> Outcome<()> {
        let s = match value {
            PolicyTemplateAllowedDefaultValues::Stringable(s) => s,
            val => {
                return Err(ValidationError::TypeMismatch {
                    expected: "String".into(),
                    got: format!("{:?}", val),
                }
                .into_errors())
            }
        };

        if let Some(min) = restrictions.min_length {
            if s.len() < min {
                return Err(ValidationError::MinLength { length: s.len(), min }.into_errors());
            }
        }
        if let Some(max) = restrictions.max_length {
            if s.len() > max {
                return Err(ValidationError::MaxLength { length: s.len(), max }.into_errors());
            }
        }
        if let Some(pattern) = &restrictions.regex {
            let re = Regex::new(pattern)
                .map_err(|e| Errors::crazy(format!("Invalid regex: {}", e), Some(Box::new(e))))?;
            if !re.is_match(s) {
                return Err(ValidationError::RegexMismatch { value: s.clone() }.into_errors());
            }
        }
        Ok(())
    }
}

pub struct NumericValidator;

impl ParameterValidator for NumericValidator {
    fn validate(
        &self,
        value: &PolicyTemplateAllowedDefaultValues,
        restrictions: &ValidationRestrictions,
    ) -> Outcome<()> {
        let val = match value {
            PolicyTemplateAllowedDefaultValues::Numerable(n) => *n as f64,
            val => {
                return Err(ValidationError::TypeMismatch {
                    expected: "Numeric".into(),
                    got: format!("{:?}", val),
                }
                .into_errors())
            }
        };

        if let Some(min) = restrictions.min_value {
            if val < min {
                return Err(ValidationError::MinValue { value: val, min }.into_errors());
            }
        }
        if let Some(max) = restrictions.max_value {
            if val > max {
                return Err(ValidationError::MaxValue { value: val, max }.into_errors());
            }
        }
        Ok(())
    }
}

pub struct SelectionValidator;

impl ParameterValidator for SelectionValidator {
    fn validate(
        &self,
        value: &PolicyTemplateAllowedDefaultValues,
        restrictions: &ValidationRestrictions,
    ) -> Outcome<()> {
        let val_str = match value {
            PolicyTemplateAllowedDefaultValues::Stringable(s) => s.clone(),
            PolicyTemplateAllowedDefaultValues::Numerable(n) => n.to_string(),
        };

        if let Some(allowed) = &restrictions.values {
            let is_allowed = match allowed {
                SelectionAllowedValues::Simple(vec) => vec.contains(&val_str),
                SelectionAllowedValues::Complex(vec) => vec.iter().any(|opt| opt.value == val_str),
            };

            if !is_allowed {
                return Err(ValidationError::InvalidSelection { value: val_str }.into_errors());
            }
        }
        Ok(())
    }
}

pub struct DateTimeValidator;

impl ParameterValidator for DateTimeValidator {
    fn validate(
        &self,
        value: &PolicyTemplateAllowedDefaultValues,
        restrictions: &ValidationRestrictions,
    ) -> Outcome<()> {
        let s = match value {
            PolicyTemplateAllowedDefaultValues::Stringable(s) => s,
            val => {
                return Err(ValidationError::TypeMismatch {
                    expected: "ISO8601 String".into(),
                    got: format!("{:?}", val),
                }
                .into_errors())
            }
        };

        let input_date = chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| ValidationError::DateConfigError(e.to_string()).into_errors())?;

        if let Some(min_str) = &restrictions.min_date {
            let min_date = chrono::DateTime::parse_from_rfc3339(min_str)
                .map_err(|e| ValidationError::DateConfigError(e.to_string()).into_errors())?;

            if input_date < min_date {
                return Err(ValidationError::MinDate { value: s.clone(), min: min_str.clone() }
                    .into_errors());
            }
        }

        if let Some(max_str) = &restrictions.max_date {
            let max_date = chrono::DateTime::parse_from_rfc3339(max_str)
                .map_err(|e| ValidationError::DateConfigError(e.to_string()).into_errors())?;

            if input_date > max_date {
                return Err(ValidationError::MaxDate { value: s.clone(), max: max_str.clone() }
                    .into_errors());
            }
        }

        Ok(())
    }
}

pub struct ValidatorFactory;

impl ValidatorFactory {
    pub fn get_validator(data_type: ParameterDataType) -> Box<dyn ParameterValidator> {
        match data_type {
            ParameterDataType::String => Box::new(StringValidator),
            ParameterDataType::Integer | ParameterDataType::Float => Box::new(NumericValidator),
            ParameterDataType::Selection => Box::new(SelectionValidator),
            ParameterDataType::DateTime | ParameterDataType::Date | ParameterDataType::Time => {
                Box::new(DateTimeValidator)
            }
            _ => Box::new(StringValidator),
        }
    }
}
