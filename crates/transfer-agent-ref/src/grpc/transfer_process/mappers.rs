/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::batch_requests::BatchRequests;
use serde_json::Value as Json;
use tonic::Status;
use urn::Urn;

use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::ids::TenantId;
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, EditTransferProcessRequest,
    ListTransferProcessesRequest, ProtocolId as ProtoProtocolId, TransferCorrelation as ProtoCorrelation,
    TransferProcessListResponse, TransferProcessResponse, TransferRole as ProtoTransferRole,
};
use crate::services::transfer_process::views::TransferProcessView;

// ─── Request → Domain ───────────────────────────────────────────────────────

pub fn into_list_params(
    req: ListTransferProcessesRequest,
    tenant_id: Option<TenantId>,
) -> Result<(TransferProcessFilter, Page, Sort), Status> {
    let protocol = non_empty(&req.protocol).map(parse_protocol_id).transpose()?;
    let state = non_empty(&req.state).map(|s| ProtocolState(s.into()));
    let role = non_empty(&req.role).map(parse_role_str).transpose()?;
    let agreement_id = non_empty(&req.agreement_id)
        .map(|s| parse_urn(s, "agreement_id"))
        .transpose()?;
    let peer_participant_id = non_empty(&req.peer_participant_id)
        .map(|s| {
            use crate::entities::ids::ParticipantId;
            parse_urn(s, "peer_participant_id").map(ParticipantId::new)
        })
        .transpose()?;
    let created_after = non_empty(&req.created_after)
        .map(|s| parse_dt(s, "created_after"))
        .transpose()?;
    let created_before = non_empty(&req.created_before)
        .map(|s| parse_dt(s, "created_before"))
        .transpose()?;

    let filter = TransferProcessFilter {
        tenant_id,
        protocol,
        state,
        role,
        agreement_id,
        peer_participant_id,
        created_after,
        created_before,
    };
    let cursor = non_empty(&req.cursor).map(|s| s.to_owned());
    let page = Page { limit: if req.limit == 0 { 20 } else { req.limit }, cursor };
    let sort = non_empty(&req.sort).map(parse_sort).transpose()?.unwrap_or_default();
    Ok((filter, page, sort))
}

pub fn into_batch(req: BatchTransferProcessesRequest) -> Result<BatchRequests, Status> {
    let ids = req
        .ids
        .iter()
        .map(|s| parse_urn(s, "ids"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BatchRequests { ids })
}

pub fn into_create_cmd(
    req: CreateTransferProcessRequest,
    tenant_id: TenantId,
) -> Result<NewTransferProcessCommand, Status> {
    use crate::entities::ids::ParticipantId;
    use url::Url;

    let role = parse_proto_role(req.role)?;
    let protocol = parse_proto_protocol_id(req.protocol)?;
    let agreement_id = parse_urn(&req.agreement_id, "agreement_id")?;
    let peer_participant_id =
        ParticipantId::new(parse_urn(&req.peer_participant_id, "peer_participant_id")?);
    let callback_address = non_empty(&req.callback_address)
        .map(|s| {
            Url::from_str(s).map_err(|e| Status::invalid_argument(format!("callback_address: {e}")))
        })
        .transpose()?;
    let identifiers = if req.identifiers.is_empty() { None } else { Some(req.identifiers) };
    let properties = non_empty(&req.properties)
        .map(|s| {
            serde_json::from_str::<Json>(s)
                .map_err(|e| Status::invalid_argument(format!("properties: {e}")))
        })
        .transpose()?;
    let initial_state_metadata = StateMetadata {
        attribute: non_empty(&req.initial_state_attribute).map(|s| s.to_owned()),
        reason: if req.initial_state_reasons.is_empty() { None } else { Some(req.initial_state_reasons) },
        code: non_empty(&req.initial_state_code).map(|s| s.to_owned()),
    };

    Ok(NewTransferProcessCommand {
        id: None,
        tenant_id: Some(tenant_id),
        role,
        protocol,
        initial_state: ProtocolState(req.initial_state.into()),
        initial_state_metadata,
        callback_address,
        connector_instance_id: None,
        agreement_id,
        peer_participant_id,
        identifiers,
        properties,
    })
}

pub fn into_edit_cmd(req: EditTransferProcessRequest) -> Result<EditTransferProcessCommand, Status> {
    let state = non_empty(&req.state).map(|s| ProtocolState(s.into()));
    let state_metadata = if non_empty(&req.state_attribute).is_some()
        || non_empty(&req.state_code).is_some()
        || !req.state_reasons.is_empty()
    {
        Some(StateMetadata {
            attribute: non_empty(&req.state_attribute).map(|s| s.to_owned()),
            reason: if req.state_reasons.is_empty() { None } else { Some(req.state_reasons) },
            code: non_empty(&req.state_code).map(|s| s.to_owned()),
        })
    } else {
        None
    };
    let identifiers = if req.identifiers.is_empty() { None } else { Some(req.identifiers) };
    let properties = non_empty(&req.properties)
        .map(|s| {
            serde_json::from_str::<Json>(s)
                .map_err(|e| Status::invalid_argument(format!("properties: {e}")))
        })
        .transpose()?;
    let error_details = non_empty(&req.error_details)
        .map(|s| {
            serde_json::from_str::<Json>(s)
                .map_err(|e| Status::invalid_argument(format!("error_details: {e}")))
        })
        .transpose()?;

    Ok(EditTransferProcessCommand { state, state_metadata, identifiers, properties, error_details })
}

// ─── Domain → Response ──────────────────────────────────────────────────────

pub fn from_view(view: TransferProcessView) -> TransferProcessResponse {
    let role = domain_role_to_proto(view.role) as i32;
    let protocol = domain_protocol_to_proto(&view.protocol) as i32;
    let state_metadata = Some(from_state_metadata(view.state_metadata));
    let correlation = Some(from_correlation(view.correlation));
    let properties = serde_json::to_string(&view.properties).unwrap_or_default();
    let error_details = view
        .error_details
        .map(|v| serde_json::to_string(&v).unwrap_or_default())
        .unwrap_or_default();

    TransferProcessResponse {
        id: view.id.to_string(),
        tenant_id: view.tenant_id.to_string(),
        role,
        protocol,
        state: view.state.0.to_string(),
        state_metadata,
        correlation,
        properties,
        error_details,
        created_at: view.created_at.to_rfc3339(),
        updated_at: view.updated_at.to_rfc3339(),
        version: view.version,
    }
}

pub fn from_paginated(result: Paginated<TransferProcessView>) -> TransferProcessListResponse {
    TransferProcessListResponse {
        items: result.items.into_iter().map(from_view).collect(),
        next_cursor: result.next_cursor.unwrap_or_default(),
        total: result.total.unwrap_or(0),
    }
}

pub fn from_vec(views: Vec<TransferProcessView>) -> TransferProcessListResponse {
    TransferProcessListResponse {
        items: views.into_iter().map(from_view).collect(),
        next_cursor: String::new(),
        total: 0,
    }
}

// ─── Nested type conversions ─────────────────────────────────────────────────

fn from_state_metadata(meta: StateMetadata) -> crate::grpc::api::transfer_processes::StateMetadata {
    crate::grpc::api::transfer_processes::StateMetadata {
        attribute: meta.attribute.unwrap_or_default(),
        reason: meta.reason.unwrap_or_default(),
        code: meta.code.unwrap_or_default(),
    }
}

fn from_correlation(corr: TransferCorrelation) -> ProtoCorrelation {
    ProtoCorrelation {
        identifiers: corr.identifiers,
        consumer_pid: corr.consumer_pid.unwrap_or_default(),
        provider_pid: corr.provider_pid.unwrap_or_default(),
        agreement_id: corr.agreement_id.map(|u| u.to_string()).unwrap_or_default(),
        callback_address: corr.callback_address.map(|u| u.to_string()).unwrap_or_default(),
        peer_participant_id: corr.peer_participant_id.map(|p| p.to_string()).unwrap_or_default(),
    }
}

// ─── Enum conversions ────────────────────────────────────────────────────────

fn parse_proto_role(value: i32) -> Result<TransferRole, Status> {
    match ProtoTransferRole::try_from(value) {
        Ok(ProtoTransferRole::Provider) => Ok(TransferRole::Provider),
        Ok(ProtoTransferRole::Consumer) => Ok(TransferRole::Consumer),
        Ok(ProtoTransferRole::Relay) => Ok(TransferRole::Relay),
        Err(_) => Err(Status::invalid_argument(format!("unknown TransferRole: {value}"))),
    }
}

fn parse_role_str(s: &str) -> Result<TransferRole, Status> {
    match s {
        "provider" => Ok(TransferRole::Provider),
        "consumer" => Ok(TransferRole::Consumer),
        "relay" => Ok(TransferRole::Relay),
        other => Err(Status::invalid_argument(format!("unknown role: {other}"))),
    }
}

fn parse_proto_protocol_id(value: i32) -> Result<ProtocolId, Status> {
    match ProtoProtocolId::try_from(value) {
        Ok(ProtoProtocolId::Dsp2024) => Ok(ProtocolId::Dsp2024),
        Ok(ProtoProtocolId::Dsp20251) => Ok(ProtocolId::Dsp2025_1),
        Err(_) => Err(Status::invalid_argument(format!("unknown ProtocolId: {value}"))),
    }
}

fn parse_protocol_id(s: &str) -> Result<ProtocolId, Status> {
    match s {
        "dsp2024" => Ok(ProtocolId::Dsp2024),
        "dsp2025_1" => Ok(ProtocolId::Dsp2025_1),
        other => Err(Status::invalid_argument(format!("unknown protocol: {other}"))),
    }
}

fn domain_role_to_proto(role: TransferRole) -> ProtoTransferRole {
    match role {
        TransferRole::Provider => ProtoTransferRole::Provider,
        TransferRole::Consumer => ProtoTransferRole::Consumer,
        TransferRole::Relay => ProtoTransferRole::Relay,
    }
}

fn domain_protocol_to_proto(protocol: &ProtocolId) -> ProtoProtocolId {
    match protocol {
        ProtocolId::Dsp2024 => ProtoProtocolId::Dsp2024,
        ProtocolId::Dsp2025_1 => ProtoProtocolId::Dsp20251,
    }
}

// ─── Pagination / sort ──────────────────────────────────────────────────────

fn parse_sort(s: &str) -> Result<Sort, Status> {
    match s {
        "created_at_asc" => Ok(Sort::CreatedAtAsc),
        "created_at_desc" => Ok(Sort::CreatedAtDesc),
        "updated_at_desc" => Ok(Sort::UpdatedAtDesc),
        other => Err(Status::invalid_argument(format!("unknown sort: {other}"))),
    }
}

// ─── Primitives ──────────────────────────────────────────────────────────────

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
