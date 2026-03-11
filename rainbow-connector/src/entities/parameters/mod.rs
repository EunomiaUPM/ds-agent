//! Parameter extraction, enrichment, and resolution pipeline.
//!
//! This module implements a robust, multi-phase pipeline that transforms a
//! parameterised connector template into a fully resolved instance configuration.
//!
//! # The Processing Pipeline
//!
//! Instance parameters flow through three ordered phases:
//!
//! ```text
//! HashMap<String, Value>   (user-supplied instance parameters, mutable)
//!        │
//!        ├─ Phase 1a: SysParameterEnricher
//!        │   Scans the template for {{__SYS_*__}} placeholders and injects
//!        │   the corresponding runtime values (URN, token, timestamp …).
//!        │
//!        ├─ Phase 1b: DefaultParameterEnricher
//!        │   Fills in default values for parameters that the user left unset.
//!        │
//!        └─ Phase 2: TemplateParametersResolver (read-only borrow)
//!            Walks ConnectorTemplateDto and replaces every {{__PARAM__}}
//!            placeholder with the value from the now-complete parameter map.
//! ```
//!
//! # Architecture: The Visitor Pattern
//!
//! Processing templates involves traversing a complex nested structure (auth,
//! interaction, protocols). To avoid duplicating this traversal logic, the
//! system uses a decoupled visitor pattern:
//!
//! - **Structural Walking** ([`template_walker`]): Defines *how* to traverse
//!   the `ConnectorTemplateDto` tree. This is the single source of truth for
//!   template structure.
//! - **Behaviors**: Traits that define *what* to do when a templatable field
//!   is encountered ([`ParameterExtractorBehavior`], [`ParameterResolverBehavior`]).
//! - **Visitors**: Bridges that connect a walker to a behavior
//!   ([`ParameterExtractorVisitor`], [`TemplateResolverVisitor`]).
//!
//! This decoupling allows us to add new protocols (e.g., GraphQL, MQTT) by
//! updating only the walker, while extraction and resolution logic remains
//! unchanged.
//!
//! # Sub-module overview
//!
//! | Module | Role |
//! |---|---|
//! | [`parameters`] | Parameter and template-field type definitions. |
//! | [`sys_parameter_enricher`] | Resolves `SYS_*` placeholders to runtime values. |
//! | [`default_parameter_injector`] | Injects declared default values for absent params. |
//! | [`template_parameters_resolver`] | Resolves all `{{__PARAM__}}` placeholders in a template. |
//! | [`template_parameters_extractor`] | Core logic for finding `{{__NAME__}}` tokens. |
//! | [`template_parameters_visitor`] | Read-only visitor for parameter discovery. |
//! | [`template_resolver_visitor`] | Mutable visitor for in-place resolution. |
//! | [`instance_parameters_validator`] | Validates user params against template definitions. |
//! | [`template_parameters_validator`] | Validates template placeholder/definition consistency. |
//!
//! [`ConnectorTemplateDto`]: crate::entities::connector_template::ConnectorTemplateDto
//! [`ParameterExtractorBehavior`]: template_parameters_extractor::ParameterExtractorBehavior
//! [`ParameterResolverBehavior`]: template_parameters_resolver::ParameterResolverBehavior

pub mod parameters;
pub use parameters::*;

use serde_json::Value;
use std::collections::HashMap;

pub(crate) mod default_parameter_enricher;
pub(crate) mod default_parameter_injector;
pub(crate) mod instance_parameters_validator;
pub(crate) mod sys_parameter_enricher;
pub(crate) mod system_parameter_extractor;
pub(crate) mod template_parameters_extractor;
pub mod template_parameters_resolver;
pub(crate) mod template_parameters_validator;
pub(crate) mod template_parameters_visitor;
pub(crate) mod template_resolver_visitor;
pub(crate) mod template_walker;

/// Common interface for all pipeline steps that mutate the instance parameter
/// map before template resolution.
///
/// Implementors receive a `&mut HashMap<String, Value>` representing the
/// current parameter state and may insert, but should never remove or
/// unconditionally overwrite, existing entries.
///
/// # Pipeline order
///
/// Steps must be applied in this order:
///
/// 1. [`SysParameterEnricher`] — injects `SYS_*` runtime values.
/// 2. [`DefaultParameterEnricher`] — fills in declared default values.
///
/// [`SysParameterEnricher`]: sys_parameter_enricher::SysParameterEnricher
/// [`DefaultParameterEnricher`]: default_parameter_enricher::DefaultParameterEnricher
pub trait ParameterEnricher {
    /// Enrich `params` in-place.
    ///
    /// Implementations should use [`HashMap::entry`] semantics (insert only
    /// when the key is absent) so that earlier pipeline steps are not
    /// overwritten by later ones.
    fn enrich(&self, params: &mut HashMap<String, Value>) -> anyhow::Result<()>;
}

/// A borrowed reference to a single templatable field of any supported type.
///
/// `TemplateField` is the token type passed from the visitor to the extractor.
/// Wrapping a reference (rather than owning the value) avoids cloning the DTO
/// during the traversal.
#[derive(Debug)]
pub enum TemplateField<'a> {
    TemplateString(&'a TemplateString),
    TemplateInt(&'a TemplateInt),
    TemplateBoolean(&'a TemplateBoolean),
    TemplateVecString(&'a TemplateVecString),
    TemplateMapString(&'a TemplateMapString),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoundParameterType {
    String,
    Int,
    Boolean,
    VecString,
    MapString,
}
