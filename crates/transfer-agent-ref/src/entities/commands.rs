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

use crate::entities::ids::{
    CorrelationId, MessageId, ParticipantId, RequestId, TenantId, TransferProcessId,
};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{
    ProtocolId, ProtocolMessageType, ProtocolState, StateMetadata, TransferDirection, TransferRole,
};
use crate::entities::transfer_message::MessageEnvelope;
use serde::Deserialize;
use serde_json::Value as Json;
use std::collections::HashMap;
use url::Url;
use urn::Urn;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewTransferProcessCommand {
    pub id: Option<TransferProcessId>,
    pub tenant_id: TenantId,
    pub role: TransferRole,
    pub protocol: ProtocolId,
    pub transfer_direction: TransferDirection,
    pub initial_state: ProtocolState,
    pub initial_state_metadata: StateMetadata,
    pub callback_address: Option<Url>,
    pub connector_instance_id: Option<String>,
    pub agreement_id: Urn,
    pub peer_participant_id: ParticipantId,
    pub identifiers: Option<HashMap<String, String>>,
    pub properties: Option<Json>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditTransferProcessCommand {
    pub state: Option<ProtocolState>,
    pub state_metadata: Option<StateMetadata>,
    pub identifiers: Option<HashMap<String, String>>,
    pub properties: Option<Json>,
    pub error_details: Option<Json>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewTransferMessageCommand {
    pub id: Option<MessageId>,
    pub transfer_process_id: TransferProcessId,
    pub tenant_id: TenantId,
    pub direction: Direction,
    pub protocol: ProtocolId,
    pub message_type: ProtocolMessageType,
    pub protocol_version: Option<String>,
    pub state_transition_from: ProtocolState,
    pub state_transition_to: ProtocolState,
    pub envelope: MessageEnvelope,
    pub peer_participant_id: Option<ParticipantId>,
    pub correlation_id: Option<CorrelationId>,
    pub request_id: Option<RequestId>,
}
