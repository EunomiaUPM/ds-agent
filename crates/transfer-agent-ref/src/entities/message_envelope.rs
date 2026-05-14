use chrono::{DateTime, Utc};
use urn::Urn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Direction {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone)]
pub(crate) struct MessageEnvelopeRef {
    pub id: Urn,
    pub direction: Direction,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub recorded_at: DateTime<Utc>,
}

impl MessageEnvelopeRef {
    pub fn new(
        id: Urn,
        direction: Direction,
        message_type: impl Into<String>,
        state_transition_from: impl Into<String>,
        state_transition_to: impl Into<String>,
    ) -> Self {
        Self {
            id,
            direction,
            message_type: message_type.into(),
            state_transition_from: state_transition_from.into(),
            state_transition_to: state_transition_to.into(),
            recorded_at: Utc::now(),
        }
    }
}