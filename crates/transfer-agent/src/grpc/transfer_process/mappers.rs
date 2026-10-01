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

//! Proto ⇄ domain mappers for the transfer-process RPCs.

use std::collections::HashMap;

use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::filters::TransferProcessFilter;
use crate::entities::ids::ParticipantId;
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use crate::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, EditTransferProcessRequest,
    ListTransferProcessesRequest, ProtocolId as ProtoProtocolId,
    StateMetadata as ProtoStateMetadata, TransferCorrelation as ProtoCorrelation,
    TransferProcessListResponse, TransferProcessResponse, TransferRole as ProtoTransferRole,
};
use crate::services::transfer_process::views::TransferProcessView;
use common::batch_requests::BatchRequests;
use common::grpc::{
    InvalidField, JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoEnum, ProtoField,
    ProtoFieldList,
};
use common::query::Paginated;
use tonic::Status;
use url::Url;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListTransferProcessesRequest> for ListParams<TransferProcessFilter> {
    type Error = Status;

    fn try_from(req: ListTransferProcessesRequest) -> Result<Self, Status> {
        let filter = TransferProcessFilter {
            tenant_id: None,
            protocol: req.protocol.opt_parsed::<ProtocolId>("protocol")?,
            state: req.state.non_empty().map(|s| ProtocolState(s.into())),
            role: req.role.opt_parsed::<TransferRole>("role")?,
            agreement_id: req.agreement_id.opt_urn("agreement_id")?,
            peer_participant_id: req
                .peer_participant_id
                .opt_urn("peer_participant_id")?
                .map(ParticipantId::new),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<BatchTransferProcessesRequest> for BatchRequests {
    type Error = Status;

    fn try_from(req: BatchTransferProcessesRequest) -> Result<Self, Status> {
        Ok(Self {
            ids: req.ids.urns("ids")?,
        })
    }
}

impl TryFrom<CreateTransferProcessRequest> for NewTransferProcessCommand {
    type Error = Status;

    fn try_from(req: CreateTransferProcessRequest) -> Result<Self, Status> {
        let callback_address = req
            .callback_address
            .non_empty()
            .map(|s| Url::parse(s).map_err(|e| InvalidField::status("callback_address", e)))
            .transpose()?;
        Ok(Self {
            id: None,
            tenant_id: None,
            role: req.role.proto_enum::<ProtoTransferRole>("role")?.into(),
            protocol: req
                .protocol
                .proto_enum::<ProtoProtocolId>("protocol")?
                .into(),
            initial_state: ProtocolState(req.initial_state.into()),
            initial_state_metadata: StateMetadata::from(ProtoStateMetadata {
                attribute: req.initial_state_attribute,
                reason: req.initial_state_reasons,
                code: req.initial_state_code,
            }),
            callback_address,
            connector_instance_id: None,
            agreement_id: req.agreement_id.urn("agreement_id")?,
            peer_participant_id: ParticipantId::new(
                req.peer_participant_id.urn("peer_participant_id")?,
            ),
            identifiers: Self::opt_identifiers(req.identifiers),
            properties: req.properties.map(JsonStructExt::into_json),
        })
    }
}

impl NewTransferProcessCommand {
    /// Proto maps cannot signal absence, so an empty map means "not provided".
    fn opt_identifiers(ids: HashMap<String, String>) -> Option<HashMap<String, String>> {
        (!ids.is_empty()).then_some(ids)
    }
}

impl TryFrom<EditTransferProcessRequest> for EditTransferProcessCommand {
    type Error = Status;

    fn try_from(req: EditTransferProcessRequest) -> Result<Self, Status> {
        let metadata = ProtoStateMetadata {
            attribute: req.state_attribute,
            reason: req.state_reasons,
            code: req.state_code,
        };
        let has_metadata = !metadata.attribute.is_empty()
            || !metadata.code.is_empty()
            || !metadata.reason.is_empty();
        Ok(Self {
            state: req.state.non_empty().map(|s| ProtocolState(s.into())),
            state_metadata: has_metadata.then(|| StateMetadata::from(metadata)),
            identifiers: NewTransferProcessCommand::opt_identifiers(req.identifiers),
            properties: req.properties.map(JsonStructExt::into_json),
            error_details: req.error_details.map(JsonStructExt::into_json),
        })
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<TransferProcessView> for TransferProcessResponse {
    fn from(view: TransferProcessView) -> Self {
        Self {
            id: view.id.to_string(),
            tenant_id: view.tenant_id,
            role: ProtoTransferRole::from(view.role) as i32,
            protocol: ProtoProtocolId::from(view.protocol) as i32,
            state: view.state.0.to_string(),
            state_metadata: Some(view.state_metadata.into()),
            correlation: Some(view.correlation.into()),
            properties: Some(view.properties.into_prost_struct()),
            error_details: view.error_details.map(JsonValueExt::into_prost_struct),
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
            version: view.version,
        }
    }
}

impl From<Paginated<TransferProcessView>> for TransferProcessListResponse {
    fn from(p: Paginated<TransferProcessView>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<TransferProcessView>> for TransferProcessListResponse {
    fn from(views: Vec<TransferProcessView>) -> Self {
        Self {
            total: views.len() as u64,
            items: views.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}

// Nested types ────────────────────────────────────────────────────────────

impl From<ProtoStateMetadata> for StateMetadata {
    fn from(meta: ProtoStateMetadata) -> Self {
        Self {
            attribute: meta.attribute.non_empty().map(str::to_owned),
            reason: (!meta.reason.is_empty()).then_some(meta.reason),
            code: meta.code.non_empty().map(str::to_owned),
        }
    }
}

impl From<StateMetadata> for ProtoStateMetadata {
    fn from(meta: StateMetadata) -> Self {
        Self {
            attribute: meta.attribute.unwrap_or_default(),
            reason: meta.reason.unwrap_or_default(),
            code: meta.code.unwrap_or_default(),
        }
    }
}

impl From<TransferCorrelation> for ProtoCorrelation {
    fn from(corr: TransferCorrelation) -> Self {
        Self {
            identifiers: corr.identifiers,
            consumer_pid: corr.consumer_pid.unwrap_or_default(),
            provider_pid: corr.provider_pid.unwrap_or_default(),
            agreement_id: corr.agreement_id.map(|u| u.to_string()).unwrap_or_default(),
            callback_address: corr
                .callback_address
                .map(|u| u.to_string())
                .unwrap_or_default(),
            peer_participant_id: corr
                .peer_participant_id
                .map(|p| p.to_string())
                .unwrap_or_default(),
        }
    }
}

// Proto enums ⇄ domain enums ──────────────────────────────────────────────

impl From<ProtoTransferRole> for TransferRole {
    fn from(role: ProtoTransferRole) -> Self {
        match role {
            ProtoTransferRole::Provider => TransferRole::Provider,
            ProtoTransferRole::Consumer => TransferRole::Consumer,
            ProtoTransferRole::Relay => TransferRole::Relay,
        }
    }
}

impl From<TransferRole> for ProtoTransferRole {
    fn from(role: TransferRole) -> Self {
        match role {
            TransferRole::Provider => ProtoTransferRole::Provider,
            TransferRole::Consumer => ProtoTransferRole::Consumer,
            TransferRole::Relay => ProtoTransferRole::Relay,
        }
    }
}

impl From<ProtoProtocolId> for ProtocolId {
    fn from(protocol: ProtoProtocolId) -> Self {
        match protocol {
            ProtoProtocolId::Dsp2024 => ProtocolId::Dsp2024,
            ProtoProtocolId::Dsp20251 => ProtocolId::Dsp2025_1,
        }
    }
}

impl From<ProtocolId> for ProtoProtocolId {
    fn from(protocol: ProtocolId) -> Self {
        match protocol {
            ProtocolId::Dsp2024 => ProtoProtocolId::Dsp2024,
            ProtocolId::Dsp2025_1 => ProtoProtocolId::Dsp20251,
        }
    }
}
