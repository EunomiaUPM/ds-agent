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

//! Proto ⇄ domain mappers for the transfer-message RPCs.

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::entities::ids::TransferProcessId;
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType, ProtocolState};
use crate::entities::transfer_message::Direction;
use crate::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, Direction as ProtoDirection,
    ListTransferMessagesByProcessRequest, ListTransferMessagesRequest,
    MessageEnvelope as ProtoEnvelope, TransferMessageListResponse, TransferMessageResponse,
};
use crate::services::transfer_message::views::TransferMessageView;
use common::grpc::{JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoEnum, ProtoField};
use common::query::Paginated;
use compact_str::CompactString;
use serde_json::Value as Json;
use tonic::Status;
use urn::Urn;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListTransferMessagesRequest> for ListParams<TransferMessageFilter> {
    type Error = Status;

    fn try_from(req: ListTransferMessagesRequest) -> Result<Self, Status> {
        let filter = TransferMessageFilter {
            tenant_id: None,
            direction: req.direction.opt_parsed::<Direction>("direction")?,
            protocol: req.protocol.opt_parsed::<ProtocolId>("protocol")?,
            state_transition_to: req
                .state_transition_to
                .non_empty()
                .map(|s| ProtocolState(s.into())),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

/// List parameters scoped to one transfer process.
pub(super) struct ListByProcessParams {
    pub process_id: Urn,
    pub params: ListParams<TransferMessageFilter>,
}

impl TryFrom<ListTransferMessagesByProcessRequest> for ListByProcessParams {
    type Error = Status;

    fn try_from(req: ListTransferMessagesByProcessRequest) -> Result<Self, Status> {
        let process_id = req.process_id.urn("process_id")?;
        let params = ListParams::try_from(ListTransferMessagesRequest::from(req))?;
        Ok(Self { process_id, params })
    }
}

/// The by-process request is the plain list request plus a process id.
impl From<ListTransferMessagesByProcessRequest> for ListTransferMessagesRequest {
    fn from(req: ListTransferMessagesByProcessRequest) -> Self {
        Self {
            direction: req.direction,
            protocol: req.protocol,
            state_transition_to: req.state_transition_to,
            created_after: req.created_after,
            created_before: req.created_before,
            limit: req.limit,
            cursor: req.cursor,
            sort: req.sort,
        }
    }
}

impl TryFrom<CreateTransferMessageRequest> for NewTransferMessageCommand {
    type Error = Status;

    fn try_from(req: CreateTransferMessageRequest) -> Result<Self, Status> {
        let payload = req
            .payload
            .map(JsonStructExt::into_json)
            .unwrap_or(Json::Null);
        let canonical_form = req.canonical_form.non_empty().map(str::to_owned);
        Ok(Self {
            id: None,
            transfer_process_id: TransferProcessId::new(
                req.transfer_process_id.urn("transfer_process_id")?,
            ),
            tenant_id: None,
            direction: req
                .direction
                .proto_enum::<ProtoDirection>("direction")?
                .into(),
            protocol: req.protocol.parsed::<ProtocolId>("protocol")?,
            message_type: ProtocolMessageType(CompactString::from(req.message_type)),
            state_transition_from: ProtocolState(req.state_transition_from.into()),
            state_transition_to: ProtocolState(req.state_transition_to.into()),
            envelope: MessageEnvelope::from_canonical(payload, canonical_form),
        })
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<TransferMessageView> for TransferMessageResponse {
    fn from(view: TransferMessageView) -> Self {
        Self {
            id: view.id.to_string(),
            transfer_process_id: view.transfer_process_id.to_string(),
            tenant_id: view.tenant_id,
            direction: ProtoDirection::from(view.direction) as i32,
            protocol: view.protocol.to_string(),
            message_type: view.message_type.0.to_string(),
            state_transition_from: view.state_transition_from,
            state_transition_to: view.state_transition_to,
            envelope: Some(view.envelope.into()),
            occurred_at: view.occurred_at.to_rfc3339(),
        }
    }
}

impl From<Paginated<TransferMessageView>> for TransferMessageListResponse {
    fn from(p: Paginated<TransferMessageView>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

// Nested types ────────────────────────────────────────────────────────────

/// A null payload is sent as an absent `Struct`; the hash travels hex-encoded.
impl From<MessageEnvelope> for ProtoEnvelope {
    fn from(env: MessageEnvelope) -> Self {
        Self {
            payload: (!env.payload.is_null()).then(|| env.payload.into_prost_struct()),
            canonical_form: env.canonical_form.unwrap_or_default(),
            canonical_hash: env.canonical_hash.map(hex::encode).unwrap_or_default(),
        }
    }
}

// Proto enums ⇄ domain enums ──────────────────────────────────────────────

impl From<ProtoDirection> for Direction {
    fn from(dir: ProtoDirection) -> Self {
        match dir {
            ProtoDirection::Inbound => Direction::Inbound,
            ProtoDirection::Outbound => Direction::Outbound,
        }
    }
}

impl From<Direction> for ProtoDirection {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Inbound => ProtoDirection::Inbound,
            Direction::Outbound => ProtoDirection::Outbound,
        }
    }
}
