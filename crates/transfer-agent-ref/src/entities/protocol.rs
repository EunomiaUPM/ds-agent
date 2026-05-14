use crate::entities::ids::ParticipantId;
use compact_str::CompactString;
use std::collections::HashMap;
use urn::Urn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransferDirection {
    Push,
    Pull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransferRole {
    Provider,
    Consumer,
    Relay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProtocolId {
    Dsp2024,
    Dsp2025_1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProtocolState(pub CompactString);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProtocolMessageType(pub CompactString);

#[derive(Debug, Clone)]
pub struct StateMetadata {
    pub attribute: Option<String>,
    pub reason: Option<Vec<String>>,
    pub code: Option<String>,
}

impl StateMetadata {
    pub fn empty() -> Self {
        Self {
            attribute: None,
            reason: None,
            code: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferCorrelation {
    pub identifiers: HashMap<String, String>,
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    pub agreement_id: Option<Urn>,
    pub callback_address: Option<url::Url>,
    pub peer_participant_id: Option<ParticipantId>,
}
