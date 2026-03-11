use crate::entities::parameters::{TemplateString, TemplateVecString};
use serde::{Deserialize, Serialize};

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
