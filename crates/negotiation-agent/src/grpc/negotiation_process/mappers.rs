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

//! Mappers between proto and domain types for the negotiation-process RPCs.

use crate::entities::filters::NegotiationProcessFilter;
use crate::entities::negotiation_process::{EditNegotiationProcessDto, NewNegotiationProcessDto};
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationProcessRequest, GetBatchNegotiationProcessesRequest,
    ListNegotiationProcessesRequest, NegotiationProcess as ProtoProcess,
    NegotiationProcessListResponse, NegotiationProcessResponse, PutNegotiationProcessRequest,
};
use crate::services::negotiation_process::views::NegotiationProcessView;
use std::collections::HashMap;
use tonic::Status;
use ymir::batch_requests::BatchRequests;
use ymir::grpc::{JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoField, ProtoFieldList};
use ymir::paginated_spec::Paginated;

impl TryFrom<ListNegotiationProcessesRequest> for ListParams<NegotiationProcessFilter> {
    type Error = Status;

    fn try_from(req: ListNegotiationProcessesRequest) -> Result<Self, Status> {
        let filter = NegotiationProcessFilter {
            id: None,
            user_id: None,
            state: req.state.non_empty().map(str::to_owned),
            role: req.role.non_empty().map(str::to_owned),
            protocol: req.protocol.non_empty().map(str::to_owned),
            associated_agent_peer: req.associated_agent_peer.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<GetBatchNegotiationProcessesRequest> for BatchRequests {
    type Error = Status;

    fn try_from(req: GetBatchNegotiationProcessesRequest) -> Result<Self, Status> {
        Ok(Self {
            ids: req.ids.urns("ids")?,
        })
    }
}

impl TryFrom<CreateNegotiationProcessRequest> for NewNegotiationProcessDto {
    type Error = Status;

    fn try_from(req: CreateNegotiationProcessRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            visibility: None,
            owner: None,
            state: req.state,
            state_attribute: req.state_attribute,
            associated_agent_peer: req.associated_agent_peer,
            protocol: req.protocol,
            callback_address: req.callback_address,
            role: req.role,
            properties: req.properties.map(JsonStructExt::into_json),
            identifiers: Self::opt_identifiers(req.identifiers),
        })
    }
}

impl NewNegotiationProcessDto {
    /// Proto maps cannot signal absence, so an empty map means "not provided".
    fn opt_identifiers(ids: HashMap<String, String>) -> Option<HashMap<String, String>> {
        (!ids.is_empty()).then_some(ids)
    }
}

impl From<PutNegotiationProcessRequest> for EditNegotiationProcessDto {
    fn from(req: PutNegotiationProcessRequest) -> Self {
        Self {
            state: req.state,
            state_attribute: req.state_attribute,
            properties: req.properties.map(JsonStructExt::into_json),
            error_details: req.error_details.map(JsonStructExt::into_json),
            identifiers: NewNegotiationProcessDto::opt_identifiers(req.identifiers),
        }
    }
}

/// The view carries no sub-entities; the proto's nested messages, offers and agreement stay empty.
impl From<NegotiationProcessView> for ProtoProcess {
    fn from(view: NegotiationProcessView) -> Self {
        Self {
            id: view.id,
            state: view.state,
            state_attribute: view.state_attribute,
            associated_agent_peer: view.correlation.peer_participant_id,
            protocol: view.protocol,
            callback_address: view.correlation.callback_address,
            role: view.role,
            properties: Some(view.properties.into_prost_struct()),
            error_details: view.error_details.map(JsonValueExt::into_prost_struct),
            created_at: view.created_at.to_rfc3339(),
            updated_at: Some(view.updated_at.to_rfc3339()),
            identifiers: view.correlation.identifiers,
            messages: vec![],
            offers: vec![],
            agreement: None,
        }
    }
}

impl From<NegotiationProcessView> for NegotiationProcessResponse {
    fn from(view: NegotiationProcessView) -> Self {
        Self {
            process: Some(view.into()),
        }
    }
}

impl From<Paginated<NegotiationProcessView>> for NegotiationProcessListResponse {
    fn from(p: Paginated<NegotiationProcessView>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<NegotiationProcessView>> for NegotiationProcessListResponse {
    fn from(views: Vec<NegotiationProcessView>) -> Self {
        Self {
            total: views.len() as u64,
            items: views.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
