//! Protocol specification types for connector interactions.
//!
//! A [`ProtocolSpec`] wraps either an [`HttpSpec`] or a [`KafkaSpec`] and
//! describes the concrete transport used for a Pull data-access or a Push
//! subscribe / unsubscribe call.
//!
//! All string-typed fields in these specs support `{{__PARAM__}}` placeholders.
//!
//! Processing of these specs is handled via the [`ConnectorTemplateWalker`]
//! in the [`parameters`] module.
//!
//! [`ConnectorTemplateWalker`]: crate::entities::parameters::template_walker::ConnectorTemplateWalker
//! [`parameters`]: crate::entities::parameters

pub mod http;
pub mod kafka;

pub use http::*;
pub use kafka::*;

use serde::{Deserialize, Serialize};

/// A connector protocol: either HTTP or Kafka.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolSpec {
    Http(HttpSpec),
    Kafka(KafkaSpec),
}
