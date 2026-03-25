/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

//! Core parameter type system for connector templates.
//!
//! # Parameter definitions
//!
//! [`ParameterDefinition`] describes a single declared parameter in a connector
//! template: its name, display title, type, whether it is required, and an
//! optional default value.
//!
//! [`ParameterType`] enumerates the five supported value kinds.  [`SysParameterType`]
//! enumerates the built-in system parameters that the engine injects automatically
//! (URN, token, timestamps, own URL).
//!
//! # Template-field wrappers
//!
//! Each field in a connector spec that supports parameterisation is typed as one
//! of the `Template*` wrappers.  Every wrapper has two variants:
//!
//! - **`Template(String)`** — the field's value is a single `{{__NAME__}}`
//!   placeholder that resolves to the whole field value (type replacement).
//! - **`Value(T)`** — the field already holds a concrete value, possibly with
//!   embedded `{{__NAME__}}` placeholders that are interpolated as strings.
//!
//! | Type alias / enum | Rust target type |
//! |---|---|
//! | [`TemplateString`] | `String` |
//! | [`TemplateInt`] | `i64` |
//! | [`TemplateBoolean`] | `bool` |
//! | [`TemplateVecString`] | `Vec<String>` |
//! | [`TemplateMapString`] | `HashMap<String, String>` |
//!
//! # `TemplateMutable` and in-place resolution
//!
//! [`TemplateMutable`] is implemented for the `Template*` wrapper types that
//! appear in the active protocol specs ([`HttpSpec`], [`KafkaSpec`]).  A
//! [`ParameterResolverBehavior`] walks a struct tree and replaces placeholder strings with
//! resolved values.
//!
//! [`HttpSpec`]: crate::entities::resource::HttpSpec
//! [`KafkaSpec`]: crate::entities::resource::KafkaSpec
//! [`ParameterResolverBehavior`]: crate::entities::parameters::template_parameters_resolver::ParameterResolverBehavior

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ymir::errors::Errors;
// =============================================================================
// Parameter type declarations
// =============================================================================

/// The declared type of a connector template parameter.
///
/// Used in [`ParameterDefinition`] and checked during instance parameter
/// validation to ensure user-supplied values are compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParameterType {
    String,
    Int,
    Boolean,
    #[serde(rename = "VEC<STRING>")]
    VecString,
    #[serde(rename = "MAP<STRING,STRING>")]
    MapStringString,
}

/// The runtime-injected system parameter types.
///
/// These are resolved by [`SysParameterEnricher`] before template resolution.
/// Users must **not** declare parameters with these names — the validator
/// rejects any such declarations.
///
/// [`SysParameterEnricher`]: crate::entities::parameters::sys_parameter_enricher::SysParameterEnricher
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SysParameterType {
    /// `urn:uuid:<uuid>` — stable identifier for the connector instance.
    SysUrn,
    /// Random UUID string — changes on every invocation.
    SysToken,
    /// Unix timestamp (integer seconds since epoch).
    SysTimestamp,
    /// ISO 8601 / RFC 3339 timestamp string.
    SysIso8601,
    /// The connector's own base URL.
    ///
    /// When `host_docker_internal` is `true` the URL has `localhost` /
    /// `127.0.0.1` replaced with `host.docker.internal` so that it is
    /// reachable from inside a Docker container.
    SysOwnUrl { host_docker_internal: bool },
}

impl std::str::FromStr for SysParameterType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SYS_URN"        => Ok(Self::SysUrn),
            "SYS_TOKEN"      => Ok(Self::SysToken),
            "SYS_TIMESTAMP"  => Ok(Self::SysTimestamp),
            "SYS_ISO8601"    => Ok(Self::SysIso8601),
            "SYS_OWN_URL"    => Ok(Self::SysOwnUrl { host_docker_internal: false }),
            "SYS_OWN_URL_DOCKER" => Ok(Self::SysOwnUrl { host_docker_internal: true }),
            _ => Err(Errors::validation(format!("{} system parameter not valid", s), None)),
        }
    }
}

/// Metadata for a single declared parameter in a connector template.
///
/// The engine uses this to validate instance parameters and to fill in
/// missing values from `default_value` before resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterDefinition {
    /// Machine-readable name; used as the placeholder key (e.g. `MY_PARAM` for `{{__MY_PARAM__}}`).
    pub name: String,
    /// Human-readable display title.
    pub title: String,
    pub description: Option<String>,
    pub param_type: ParameterType,
    pub required: bool,
    /// Serialised default value for the parameter's type (e.g. `"42"` for an `Int`).
    pub default_value: Option<String>,
}

// =============================================================================
// Template-field wrapper types
// =============================================================================

/// A plain string field that may contain `{{__NAME__}}` placeholders.
pub type TemplateString = String;

/// An integer field that may be supplied as a literal or as a single placeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateInt {
    Value(i64),
    Template(String),
}

/// A boolean field that may be supplied as a literal or as a single placeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateBoolean {
    Value(bool),
    Template(String),
}

/// A string-vector field that may be either a concrete list or a single placeholder
/// that resolves to a JSON array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateVecString {
    Value(Vec<String>),
    Template(String),
}

/// A string-map field that may be either a concrete map or a single placeholder
/// that resolves to a JSON object with string values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateMapString {
    Value(HashMap<String, String>),
    Template(String),
}
