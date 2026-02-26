//! Parameter extraction and validation primitives.
//!
//! The three sub-modules collaborate as a mini-pipeline:
//!
//! | Module      | Role |
//! |-------------|------|
//! | `extractor` | Accumulates `{{__NAME__}}` placeholders found in individual fields. |
//! | `visitor`   | Walks a full [`ConnectorTemplateDto`] and feeds each templatable field to the extractor. |
//! | `validator` | Compares the accumulated names against the connector's declared [`ParameterDefinition`] list. |
//!
//! [`ConnectorTemplateDto`]: crate::entities::connector_template::ConnectorTemplateDto
//! [`ParameterDefinition`]: crate::entities::common::parameters::ParameterDefinition

use crate::entities::common::parameters::{TemplateBoolean, TemplateInt, TemplateMapString, TemplateString};
use crate::TemplateVecString;

pub(crate) mod visitor;
pub(crate) mod extractor;
pub(crate) mod validator;

/// A borrowed reference to a single templatable field of any supported type.
///
/// `TemplateField` is the token type passed from the visitor to the extractor.
/// Wrapping a reference (rather than owning the value) avoids cloning the DTO
/// during the traversal.
pub enum TemplateField<'a> {
    TemplateString(&'a TemplateString),
    TemplateInt(&'a TemplateInt),
    TemplateBoolean(&'a TemplateBoolean),
    TemplateVecString(&'a TemplateVecString),
    TemplateMapString(&'a TemplateMapString),
}
