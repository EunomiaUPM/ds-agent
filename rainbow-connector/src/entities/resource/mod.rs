//! Protocol specification types for connector interactions.
//!
//! A [`ProtocolSpec`] wraps either an [`HttpSpec`] or a [`KafkaSpec`] and
//! describes the concrete transport used for a Pull data-access or a Push
//! subscribe / unsubscribe call.
//!
//! All string-typed fields in these specs support `{{__PARAM__}}` placeholders.
//! The [`TemplateMutable`] implementations enable in-place resolution of those
//! placeholders by a [`ParameterResolverBehavior`] (e.g. `TemplateParametersResolver`).
//!
//! [`TemplateMutable`]: crate::entities::common::parameters::TemplateMutable
//! [`ParameterResolverBehavior`]: crate::entities::parameters::template_parameters_resolver::ParameterResolverBehavior

use crate::entities::parameters::{TemplateMapString, TemplateString, TemplateVecString};
use serde::{Deserialize, Serialize};

/// A connector protocol: either HTTP or Kafka.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolSpec {
    Http(HttpSpec),
    Kafka(KafkaSpec),
}

/// HTTP protocol specification.
///
/// All string fields support `{{__PARAM__}}` placeholders.  The `headers` map
/// values and `body_template` body string are each resolved individually.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpSpec {
    /// URL, possibly containing placeholders (e.g. `https://api.example.com/{{__ID__}}`).
    pub url_template: TemplateString,
    /// HTTP method(s).  Typically a single-element list such as `["GET"]`.
    pub method: TemplateVecString,
    /// Optional request headers map.  Values may contain placeholders.
    pub headers: Option<TemplateMapString>,
    /// Optional request body.  May be a JSON template string.
    pub body_template: Option<TemplateString>,
}

/// Kafka protocol specification.
///
/// All string fields support `{{__PARAM__}}` placeholders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaSpec {
    /// Broker addresses (e.g. `["localhost:9092"]`).
    pub brokers: TemplateVecString,
    /// Topic name.
    pub topic: TemplateString,
    /// Optional consumer group ID.
    pub group_id: Option<TemplateString>,
}
