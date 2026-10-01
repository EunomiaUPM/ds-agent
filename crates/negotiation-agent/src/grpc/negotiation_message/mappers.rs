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

//! Proto ⇄ domain mappers for the negotiation-message RPCs.

use crate::entities::filters::NegotiationMessageFilter;
use crate::entities::negotiation_message::NewNegotiationMessageDto;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationMessageRequest, GetMessagesByProcessIdRequest, ListNegotiationMessagesRequest,
    NegotiationMessage as ProtoMessage, NegotiationMessageListResponse, NegotiationMessageResponse,
};
use crate::services::agreement::views::AgreementView;
use crate::services::negotiation_message::views::NegotiationMessageView;
use crate::services::offer::views::OfferView;
use common::grpc::{JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoField};
use common::paginated_spec::Paginated;
use serde_json::Value as Json;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListNegotiationMessagesRequest> for ListParams<NegotiationMessageFilter> {
    type Error = Status;

    fn try_from(req: ListNegotiationMessagesRequest) -> Result<Self, Status> {
        let filter = NegotiationMessageFilter {
            id: None,
            tenant_id: None,
            process_id: req.process_id.non_empty().map(str::to_owned),
            protocol: req.protocol.non_empty().map(str::to_owned),
            message_type: req.message_type.non_empty().map(str::to_owned),
            direction: req.direction.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

/// The by-process RPC is a list scoped to one (validated) process URN.
impl TryFrom<GetMessagesByProcessIdRequest> for ListParams<NegotiationMessageFilter> {
    type Error = Status;

    fn try_from(req: GetMessagesByProcessIdRequest) -> Result<Self, Status> {
        let filter = NegotiationMessageFilter {
            process_id: Some(req.process_id.urn("process_id")?.to_string()),
            ..Default::default()
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreateNegotiationMessageRequest> for NewNegotiationMessageDto {
    type Error = Status;

    fn try_from(req: CreateNegotiationMessageRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            negotiation_agent_process_id: req
                .negotiation_agent_process_id
                .urn("negotiation_agent_process_id")?,
            direction: req.direction,
            protocol: req.protocol,
            message_type: req.message_type,
            state_transition_from: req.state_transition_from,
            state_transition_to: req.state_transition_to,
            payload: req
                .payload
                .map(JsonStructExt::into_json)
                .unwrap_or(Json::Object(Default::default())),
        })
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<NegotiationMessageView> for ProtoMessage {
    fn from(view: NegotiationMessageView) -> Self {
        let inner = view.inner;
        Self {
            id: inner.id,
            negotiation_agent_process_id: inner.negotiation_agent_process_id,
            direction: inner.direction,
            protocol: inner.protocol,
            message_type: inner.message_type,
            state_transition_from: inner.state_transition_from,
            state_transition_to: inner.state_transition_to,
            payload: Some(inner.payload.into_prost_struct()),
            created_at: inner.created_at.to_rfc3339(),
            offer: view.offer.map(|o| OfferView::assemble(o).into()),
            agreement: view.agreement.map(|a| AgreementView::assemble(a).into()),
        }
    }
}

impl From<NegotiationMessageView> for NegotiationMessageResponse {
    fn from(view: NegotiationMessageView) -> Self {
        Self {
            message: Some(view.into()),
        }
    }
}

impl From<Paginated<NegotiationMessageView>> for NegotiationMessageListResponse {
    fn from(p: Paginated<NegotiationMessageView>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}
