use crate::entities::ids::{MessageId, ParticipantId, TenantId, TransferProcessId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{
    ProtocolId, ProtocolMessageType, ProtocolState, StateMetadata, TransferDirection, TransferRole,
};
use crate::entities::transfer_message::MessageEnvelope;
use serde_json::Value as Json;
use std::collections::HashMap;
use url::Url;
use urn::Urn;

pub(crate) struct NewTransferProcessCommand {
    /// Common
    pub id: Option<TransferProcessId>,
    pub tenant_id: TenantId,
    pub role: TransferRole,
    pub protocol: ProtocolId,
    pub transfer_direction: TransferDirection,

    // Protocol
    pub initial_state: ProtocolState,
    pub initial_state_metadata: StateMetadata,
    pub callback_address: Option<Url>,

    /// Externals
    pub connector_instance_id: Option<String>,
    pub agreement_id: Urn,
    pub peer_participant_id: ParticipantId,
    pub identifiers: Option<HashMap<String, String>>,

    /// Extra
    pub properties: Option<Json>,
}

pub(crate) struct EditTransferProcessCommand {
    // state
    pub state: Option<ProtocolState>,
    pub state_metadata: Option<StateMetadata>,

    // External
    pub identifiers: Option<HashMap<String, String>>,

    // Extra
    pub properties: Option<Json>,
    pub error_details: Option<Json>,
}

pub(crate) struct NewTransferMessageCommand {
    pub id: Option<MessageId>,
    pub transfer_process_id: TransferProcessId,
    pub tenant_id: TenantId,
    pub direction: Direction,

    // Protocol
    pub protocol: ProtocolId,
    pub message_type: ProtocolMessageType,

    /// Transition
    pub state_transition_from: ProtocolState,
    pub state_transition_to: ProtocolState,

    /// Complete wire
    pub envelope: MessageEnvelope,
}
