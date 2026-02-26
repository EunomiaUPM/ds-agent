//! Internal implementation details of the connector template lifecycle.
//!
//! This module contains the processing pipeline that runs when a new connector
//! template is created or validated. The pipeline has three stages:
//!
//! ```text
//! ConnectorTemplateDto
//!        │
//!        ▼
//! ParameterExtractorVisitor   — walks every templatable field of the DTO
//!        │  feeds TemplateField tokens to ↓
//!        ▼
//! ParameterExtractor          — scans each field for {{__NAME__}} placeholders
//!        │  produces Vec<String> (found names) fed to ↓
//!        ▼
//! ParameterValidator          — checks found names against declared ParameterDefinitions
//! ```
//!
//! The entry point that wires all three stages together is
//! [`connector_instance::create_instance`].

pub(crate) mod connector_instance;
pub(crate) mod parameters;