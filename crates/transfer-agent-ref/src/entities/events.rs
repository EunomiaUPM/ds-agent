/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::ids::TransferProcessId;
use crate::entities::message_envelope::MessageEnvelopeRef;
use crate::entities::protocol::{ProtocolState, StateMetadata};
use chrono::{DateTime, Utc};

#[allow(dead_code)]
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
