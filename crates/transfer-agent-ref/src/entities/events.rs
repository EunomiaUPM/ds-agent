use crate::entities::ids::TransferProcessId;
use crate::entities::message_envelope::MessageEnvelopeRef;
use crate::entities::protocol::{ProtocolState, StateMetadata};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub(crate) enum TransferProcessEvent {
    Created {
        transfer_id: TransferProcessId,
        occurred_at: DateTime<Utc>,
    },
    StateChanged {
        transfer_id: TransferProcessId,
        new_state: ProtocolState,
        metadata: StateMetadata,
        occurred_at: DateTime<Utc>,
    },
    InboundEnvelopeRecorded {
        transfer_id: TransferProcessId,
        envelope: MessageEnvelopeRef,
        occurred_at: DateTime<Utc>,
    },
    OutboundEnvelopeRecorded {
        transfer_id: TransferProcessId,
        envelope: MessageEnvelopeRef,
        occurred_at: DateTime<Utc>,
    },
}
