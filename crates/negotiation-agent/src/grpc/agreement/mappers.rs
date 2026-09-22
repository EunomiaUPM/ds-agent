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

//! Proto ⇄ domain mappers for the agreement RPCs.

use crate::entities::agreement::{EditAgreementDto, NewAgreementDto};
use crate::entities::filters::AgreementFilter;
use crate::grpc::api::negotiation_agent::{
    Agreement as ProtoAgreement, AgreementListResponse, AgreementResponse, CreateAgreementRequest,
    GetBatchAgreementsRequest, ListAgreementsRequest, PutAgreementRequest,
};
use crate::services::agreement::views::AgreementView;
use common::batch_requests::BatchRequests;
use common::grpc::{JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoField, ProtoFieldList};
use common::paginated_spec::Paginated;
use serde_json::Value as Json;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListAgreementsRequest> for ListParams<AgreementFilter> {
    type Error = Status;

    fn try_from(req: ListAgreementsRequest) -> Result<Self, Status> {
        let filter = AgreementFilter {
            id: None,
            tenant_id: None,
            process_id: req.process_id.non_empty().map(str::to_owned),
            consumer_id: req.consumer_id.non_empty().map(str::to_owned),
            provider_id: req.provider_id.non_empty().map(str::to_owned),
            target: req.target.non_empty().map(str::to_owned),
            state: req.state.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<GetBatchAgreementsRequest> for BatchRequests {
    type Error = Status;

    fn try_from(req: GetBatchAgreementsRequest) -> Result<Self, Status> {
        Ok(Self {
            ids: req.ids.urns("ids")?,
        })
    }
}

impl TryFrom<CreateAgreementRequest> for NewAgreementDto {
    type Error = Status;

    fn try_from(req: CreateAgreementRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            negotiation_agent_process_id: req
                .negotiation_agent_process_id
                .urn("negotiation_agent_process_id")?,
            negotiation_agent_message_id: req
                .negotiation_agent_message_id
                .urn("negotiation_agent_message_id")?,
            consumer_participant_id: req.consumer_participant_id,
            provider_participant_id: req.provider_participant_id,
            agreement_content: req
                .agreement_content
                .map(JsonStructExt::into_json)
                .unwrap_or(Json::Object(Default::default())),
            target: req.target.urn("target")?,
        })
    }
}

impl From<PutAgreementRequest> for EditAgreementDto {
    fn from(req: PutAgreementRequest) -> Self {
        Self { state: req.state }
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<AgreementView> for ProtoAgreement {
    fn from(view: AgreementView) -> Self {
        let inner = view.inner;
        Self {
            id: inner.id,
            negotiation_agent_process_id: inner.negotiation_agent_process_id,
            negotiation_agent_message_id: inner.negotiation_agent_message_id,
            consumer_participant_id: inner.consumer_participant_id,
            provider_participant_id: inner.provider_participant_id,
            agreement_content: Some(inner.agreement_content.into_prost_struct()),
            target: inner.target,
            state: inner.state,
            created_at: inner.created_at.to_rfc3339(),
            updated_at: inner.updated_at.map(|d| d.to_rfc3339()),
        }
    }
}

impl From<AgreementView> for AgreementResponse {
    fn from(view: AgreementView) -> Self {
        Self {
            agreement: Some(view.into()),
        }
    }
}

impl From<Paginated<AgreementView>> for AgreementListResponse {
    fn from(p: Paginated<AgreementView>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<AgreementView>> for AgreementListResponse {
    fn from(views: Vec<AgreementView>) -> Self {
        Self {
            total: views.len() as u64,
            items: views.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
