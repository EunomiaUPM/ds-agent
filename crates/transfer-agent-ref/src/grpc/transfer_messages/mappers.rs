/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use sha2::{Digest, Sha256};
use tonic::Status;
use urn::Urn;

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::ids::{
    CorrelationId, ParticipantId, RequestId, TenantId, TransferProcessId,
};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType, ProtocolState};
use crate::entities::transfer_message::{MessageEnvelope, MessageProcessingResult};
use crate::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, Direction as ProtoDirection, ListTransferMessagesByProcessRequest,
    ListTransferMessagesRequest, MessageEnvelope as ProtoEnvelope,
    MessageProcessingResult as ProtoResult, MessageStatus, TransferMessageListResponse,
    TransferMessageResponse,
};
use crate::entities::query::{Page, Paginated, Sort, TransferMessageFilter};
use crate::services::transfer_message::views::TransferMessageView;

// ─── Request → Domain ───────────────────────────────────────────────────────

pub fn into_list_params(
    req: ListTransferMessagesRequest,
    tenant_id: Option<TenantId>,
) -> Result<(TransferMessageFilter, Page, Sort), Status> {
    let direction = non_empty(&req.direction).map(parse_direction_str).transpose()?;
    let protocol = non_empty(&req.protocol).map(parse_protocol_id).transpose()?;
    let state_transition_to = non_empty(&req.state_transition_to).map(|s| ProtocolState(s.into()));
    let created_after = non_empty(&req.created_after)
        .map(|s| parse_dt(s, "created_after"))
        .transpose()?;
    let created_before = non_empty(&req.created_before)
        .map(|s| parse_dt(s, "created_before"))
        .transpose()?;

    let filter = TransferMessageFilter {
        tenant_id,
        direction,
        protocol,
        state_transition_to,
        created_after,
        created_before,
    };
    let cursor = non_empty(&req.cursor).map(|s| s.to_owned());
    let page = Page { limit: if req.limit == 0 { 20 } else { req.limit }, cursor };
    let sort = non_empty(&req.sort).map(parse_sort).transpose()?.unwrap_or_default();
    Ok((filter, page, sort))
}

pub fn into_list_by_process_params(
    req: ListTransferMessagesByProcessRequest,
    tenant_id: Option<TenantId>,
) -> Result<(Urn, TransferMessageFilter, Page, Sort), Status> {
    let process_id = parse_urn(&req.process_id, "process_id")?;
    let direction = non_empty(&req.direction).map(parse_direction_str).transpose()?;
    let protocol = non_empty(&req.protocol).map(parse_protocol_id).transpose()?;
    let state_transition_to = non_empty(&req.state_transition_to).map(|s| ProtocolState(s.into()));
    let created_after = non_empty(&req.created_after)
        .map(|s| parse_dt(s, "created_after"))
        .transpose()?;
    let created_before = non_empty(&req.created_before)
        .map(|s| parse_dt(s, "created_before"))
        .transpose()?;

    let filter = TransferMessageFilter {
        tenant_id,
        direction,
        protocol,
        state_transition_to,
        created_after,
        created_before,
    };
    let cursor = non_empty(&req.cursor).map(|s| s.to_owned());
    let page = Page { limit: if req.limit == 0 { 20 } else { req.limit }, cursor };
    let sort = non_empty(&req.sort).map(parse_sort).transpose()?.unwrap_or_default();
    Ok((process_id, filter, page, sort))
}

pub fn into_create_cmd(
    req: CreateTransferMessageRequest,
    tenant_id: TenantId,
) -> Result<NewTransferMessageCommand, Status> {
    let process_urn = parse_urn(&req.transfer_process_id, "transfer_process_id")?;
    let direction = parse_proto_direction(req.direction)?;
    let protocol = parse_protocol_id(&req.protocol)?;
    let peer_participant_id = non_empty(&req.peer_participant_id)
        .map(|s| parse_urn(s, "peer_participant_id").map(ParticipantId::new))
        .transpose()?;
    let correlation_id = non_empty(&req.correlation_id).map(CorrelationId::new);
    let request_id = non_empty(&req.request_id).map(RequestId::new);
    let protocol_version = non_empty(&req.protocol_version).map(|s| s.to_owned());
    let envelope = build_envelope(req.raw_bytes, req.content_type, &req.headers, req.canonical_form)?;

    Ok(NewTransferMessageCommand {
        id: None,
        transfer_process_id: TransferProcessId::new(process_urn),
        tenant_id: Some(tenant_id),
        direction,
        protocol,
        message_type: ProtocolMessageType(CompactString::from(req.message_type)),
        protocol_version,
        state_transition_from: ProtocolState(req.state_transition_from.into()),
        state_transition_to: ProtocolState(req.state_transition_to.into()),
        envelope,
        peer_participant_id,
        correlation_id,
        request_id,
    })
}

// ─── Domain → Response ──────────────────────────────────────────────────────

pub fn from_view(view: TransferMessageView) -> TransferMessageResponse {
    let direction = domain_direction_to_proto(view.direction) as i32;
    let protocol = match &view.protocol {
        ProtocolId::Dsp2024 => "dsp2024".to_string(),
        ProtocolId::Dsp2025_1 => "dsp2025_1".to_string(),
    };
    let envelope = Some(from_envelope(&view.envelope));
    let processing_result = Some(from_processing_result(&view.processing_result));

    TransferMessageResponse {
        id: view.id.to_string(),
        transfer_process_id: view.transfer_process_id.to_string(),
        tenant_id: view.tenant_id.to_string(),
        direction,
        protocol,
        message_type: view.message_type.0.to_string(),
        protocol_version: view.protocol_version,
        envelope,
        occurred_at: view.occurred_at.to_rfc3339(),
        correlation_id: view.correlation_id.map(|c| c.to_string()).unwrap_or_default(),
        request_id: view.request_id.to_string(),
        peer_participant_id: view.peer_participant_id.to_string(),
        processing_result,
    }
}

pub fn from_paginated(result: Paginated<TransferMessageView>) -> TransferMessageListResponse {
    TransferMessageListResponse {
        items: result.items.into_iter().map(from_view).collect(),
        next_cursor: result.next_cursor.unwrap_or_default(),
        total: result.total.unwrap_or(0),
    }
}

// ─── Nested type conversions ─────────────────────────────────────────────────

fn bytes_to_hex(h: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(64);
    for b in h {
        write!(buf, "{b:02x}").unwrap();
    }
    buf
}

fn from_envelope(env: &MessageEnvelope) -> ProtoEnvelope {
    let content_hash = bytes_to_hex(&env.content_hash);
    let canonical_hash = env.canonical_hash.map(|h| bytes_to_hex(&h)).unwrap_or_default();
    let headers = serde_json::to_string(
        &env.headers.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<HashMap<_, _>>(),
    )
    .unwrap_or_default();

    ProtoEnvelope {
        raw_bytes: env.raw_bytes.to_vec(),
        content_type: env.content_type.to_string(),
        content_hash,
        headers,
        canonical_form: env.canonical_form.as_ref().map(|b| b.to_vec()).unwrap_or_default(),
        canonical_hash,
    }
}

fn from_processing_result(result: &MessageProcessingResult) -> ProtoResult {
    match result {
        MessageProcessingResult::Accepted { resulting_state } => ProtoResult {
            status: MessageStatus::Accepted as i32,
            resulting_state: resulting_state.0.to_string(),
            rejection_reason: String::new(),
            error_code: String::new(),
        },
        MessageProcessingResult::Rejected { reason, error_code } => ProtoResult {
            status: MessageStatus::Rejected as i32,
            resulting_state: String::new(),
            rejection_reason: reason.clone(),
            error_code: error_code.clone().unwrap_or_default(),
        },
        MessageProcessingResult::IdempotentReplay => ProtoResult {
            status: MessageStatus::IdempotentReplay as i32,
            resulting_state: String::new(),
            rejection_reason: String::new(),
            error_code: String::new(),
        },
    }
}

// ─── Envelope builder ────────────────────────────────────────────────────────

fn build_envelope(
    raw: Vec<u8>,
    content_type: String,
    headers_json: &str,
    canonical_form: Vec<u8>,
) -> Result<MessageEnvelope, Status> {
    let raw_bytes = Bytes::from(raw);
    let content_hash: [u8; 32] = Sha256::digest(&raw_bytes).into();
    let (canonical_form, canonical_hash) = if canonical_form.is_empty() {
        (None, None)
    } else {
        let cf = Bytes::from(canonical_form);
        let ch: [u8; 32] = Sha256::digest(&cf).into();
        (Some(cf), Some(ch))
    };
    let headers: BTreeMap<CompactString, String> = if headers_json.is_empty() {
        BTreeMap::new()
    } else {
        serde_json::from_str::<HashMap<String, String>>(headers_json)
            .map_err(|e| Status::invalid_argument(format!("headers: {e}")))?
            .into_iter()
            .map(|(k, v)| (CompactString::from(k), v))
            .collect()
    };

    Ok(MessageEnvelope {
        raw_bytes,
        content_type: CompactString::from(content_type),
        content_hash,
        headers,
        canonical_form,
        canonical_hash,
        signature: None,
    })
}

// ─── Enum conversions ────────────────────────────────────────────────────────

fn parse_proto_direction(value: i32) -> Result<Direction, Status> {
    match ProtoDirection::try_from(value) {
        Ok(ProtoDirection::Inbound) => Ok(Direction::Inbound),
        Ok(ProtoDirection::Outbound) => Ok(Direction::Outbound),
        Err(_) => Err(Status::invalid_argument(format!("unknown Direction: {value}"))),
    }
}

fn parse_direction_str(s: &str) -> Result<Direction, Status> {
    match s {
        "inbound" => Ok(Direction::Inbound),
        "outbound" => Ok(Direction::Outbound),
        other => Err(Status::invalid_argument(format!("unknown direction: {other}"))),
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
        other => Err(Status::invalid_argument(format!("unknown protocol: {other}"))),
    }
}

// ─── Pagination / sort ───────────────────────────────────────────────────────

fn parse_sort(s: &str) -> Result<Sort, Status> {
    match s {
        "created_at_asc" => Ok(Sort::CreatedAtAsc),
        "created_at_desc" => Ok(Sort::CreatedAtDesc),
        "updated_at_desc" => Ok(Sort::UpdatedAtDesc),
        other => Err(Status::invalid_argument(format!("unknown sort: {other}"))),
    }
}

// ─── Primitives ───────────────────────────────────────────────────────────────

/// Returns `Some(s)` if `s` is non-empty, `None` if it is `""`.
fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

fn parse_urn(s: &str, field: &str) -> Result<Urn, Status> {
    Urn::from_str(s).map_err(|e| Status::invalid_argument(format!("{field}: invalid URN — {e}")))
}

fn parse_dt(s: &str, field: &str) -> Result<DateTime<Utc>, Status> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Status::invalid_argument(format!("{field}: invalid RFC3339 — {e}")))
}
