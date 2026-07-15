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

use std::str::FromStr;

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
use crate::grpc::utils::{non_empty, parse_dt, parse_urn};
use crate::services::transfer_message::views::TransferMessageView;
use common::query::{Page, Paginated, Sort};
use compact_str::CompactString;
use serde_json::Value as Json;
use sha2::{Digest, Sha256};
use tonic::Status;
use urn::Urn;

// Request to Domain ───────────────────────────────────────────────────────

pub fn into_list_params(
    req: ListTransferMessagesRequest,
) -> Result<(TransferMessageFilter, Page, Sort), Status> {
    let direction = non_empty(&req.direction)
        .map(parse_direction_str)
        .transpose()?;
    let protocol = non_empty(&req.protocol)
        .map(parse_protocol_id)
        .transpose()?;
    let state_transition_to = non_empty(&req.state_transition_to).map(|s| ProtocolState(s.into()));
    let created_after = non_empty(&req.created_after)
        .map(|s| parse_dt(s, "created_after"))
        .transpose()?;
    let created_before = non_empty(&req.created_before)
        .map(|s| parse_dt(s, "created_before"))
        .transpose()?;

    let filter = TransferMessageFilter {
        tenant_id: None,
        direction,
        protocol,
        state_transition_to,
        created_after,
        created_before,
    };
    let cursor = non_empty(&req.cursor).map(|s| s.to_owned());
    let page = Page {
        limit: if req.limit == 0 { 20 } else { req.limit },
        cursor,
    };
    let sort = non_empty(&req.sort)
        .map(parse_sort)
        .transpose()?
        .unwrap_or_default();
    Ok((filter, page, sort))
}

pub fn into_list_by_process_params(
    req: ListTransferMessagesByProcessRequest,
) -> Result<(Urn, TransferMessageFilter, Page, Sort), Status> {
    let process_id = parse_urn(&req.process_id, "process_id")?;
    let direction = non_empty(&req.direction)
        .map(parse_direction_str)
        .transpose()?;
    let protocol = non_empty(&req.protocol)
        .map(parse_protocol_id)
        .transpose()?;
    let state_transition_to = non_empty(&req.state_transition_to).map(|s| ProtocolState(s.into()));
    let created_after = non_empty(&req.created_after)
        .map(|s| parse_dt(s, "created_after"))
        .transpose()?;
    let created_before = non_empty(&req.created_before)
        .map(|s| parse_dt(s, "created_before"))
        .transpose()?;

    let filter = TransferMessageFilter {
        tenant_id: None,
        direction,
        protocol,
        state_transition_to,
        created_after,
        created_before,
    };
    let cursor = non_empty(&req.cursor).map(|s| s.to_owned());
    let page = Page {
        limit: if req.limit == 0 { 20 } else { req.limit },
        cursor,
    };
    let sort = non_empty(&req.sort)
        .map(parse_sort)
        .transpose()?
        .unwrap_or_default();
    Ok((process_id, filter, page, sort))
}

pub fn into_create_cmd(
    req: CreateTransferMessageRequest,
) -> Result<NewTransferMessageCommand, Status> {
    let process_urn = parse_urn(&req.transfer_process_id, "transfer_process_id")?;
    let direction = parse_proto_direction(req.direction)?;
    let protocol = parse_protocol_id(&req.protocol)?;
    let envelope = build_envelope(req.payload, req.canonical_form)?;

    Ok(NewTransferMessageCommand {
        id: None,
        transfer_process_id: TransferProcessId::new(process_urn),
        tenant_id: None,
        direction,
        protocol,
        message_type: ProtocolMessageType(CompactString::from(req.message_type)),
        state_transition_from: ProtocolState(req.state_transition_from.into()),
        state_transition_to: ProtocolState(req.state_transition_to.into()),
        envelope,
    })
}

// Domain to Response ──────────────────────────────────────────────────────

pub fn from_view(view: TransferMessageView) -> TransferMessageResponse {
    let direction = domain_direction_to_proto(view.direction) as i32;
    let protocol = match &view.protocol {
        ProtocolId::Dsp2024 => "dsp2024".to_string(),
        ProtocolId::Dsp2025_1 => "dsp2025_1".to_string(),
    };
    let envelope = Some(from_envelope(&view.envelope));

    TransferMessageResponse {
        id: view.id.to_string(),
        transfer_process_id: view.transfer_process_id.to_string(),
        tenant_id: view.tenant_id.to_string(),
        direction,
        protocol,
        message_type: view.message_type.0.to_string(),
        state_transition_from: view.state_transition_from,
        state_transition_to: view.state_transition_to,
        envelope,
        occurred_at: view.occurred_at.to_rfc3339(),
    }
}

pub fn from_paginated(result: Paginated<TransferMessageView>) -> TransferMessageListResponse {
    TransferMessageListResponse {
        items: result.items.into_iter().map(from_view).collect(),
        next_cursor: result.next_cursor.unwrap_or_default(),
        total: result.total.unwrap_or(0),
    }
}

// Nested type conversions ─────────────────────────────────────────────────

fn bytes_to_hex(h: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(64);
    for b in h {
        write!(buf, "{b:02x}").unwrap();
    }
    buf
}

fn from_envelope(env: &MessageEnvelope) -> ProtoEnvelope {
    let canonical_hash = env
        .canonical_hash
        .map(|h| bytes_to_hex(&h))
        .unwrap_or_default();

    ProtoEnvelope {
        payload: serde_json::to_string(&env.payload).unwrap_or_default(),
        canonical_form: env.canonical_form.clone().unwrap_or_default(),
        canonical_hash,
    }
}

// Envelope builder ────────────────────────────────────────────────────────

fn build_envelope(payload_json: String, canonical_form: String) -> Result<MessageEnvelope, Status> {
    let payload: Json = if payload_json.is_empty() {
        Json::Null
    } else {
        serde_json::from_str(&payload_json)
            .map_err(|e| Status::invalid_argument(format!("payload: {e}")))?
    };
    let (canonical_form, canonical_hash) = if canonical_form.is_empty() {
        (None, None)
    } else {
        let ch: [u8; 32] = Sha256::digest(canonical_form.as_bytes()).into();
        (Some(canonical_form), Some(ch))
    };

    Ok(MessageEnvelope {
        canonical_form,
        canonical_hash,
        payload,
    })
}

// Enum conversions ────────────────────────────────────────────────────────

fn parse_proto_direction(value: i32) -> Result<Direction, Status> {
    match ProtoDirection::try_from(value) {
        Ok(ProtoDirection::Inbound) => Ok(Direction::Inbound),
        Ok(ProtoDirection::Outbound) => Ok(Direction::Outbound),
        Err(_) => Err(Status::invalid_argument(format!(
            "unknown Direction: {value}"
        ))),
    }
}

fn parse_direction_str(s: &str) -> Result<Direction, Status> {
    match s {
        "inbound" => Ok(Direction::Inbound),
        "outbound" => Ok(Direction::Outbound),
        other => Err(Status::invalid_argument(format!(
            "unknown direction: {other}"
        ))),
    }
}

fn domain_direction_to_proto(dir: Direction) -> ProtoDirection {
    match dir {
        Direction::Inbound => ProtoDirection::Inbound,
        Direction::Outbound => ProtoDirection::Outbound,
    }
}

fn parse_protocol_id(s: &str) -> Result<ProtocolId, Status> {
    match s {
        "dsp2024" => Ok(ProtocolId::Dsp2024),
        "dsp2025_1" => Ok(ProtocolId::Dsp2025_1),
        other => Err(Status::invalid_argument(format!(
            "unknown protocol: {other}"
        ))),
    }
}

// Pagination / sort ───────────────────────────────────────────────────────

fn parse_sort(s: &str) -> Result<Sort, Status> {
    match s {
        "created_at_asc" => Ok(Sort::CreatedAtAsc),
        "created_at_desc" => Ok(Sort::CreatedAtDesc),
        "updated_at_desc" => Ok(Sort::UpdatedAtDesc),
        other => Err(Status::invalid_argument(format!("unknown sort: {other}"))),
    }
}
