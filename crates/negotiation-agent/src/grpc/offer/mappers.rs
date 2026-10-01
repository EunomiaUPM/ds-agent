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

//! Proto ⇄ domain mappers for the offer RPCs.

use crate::entities::filters::OfferFilter;
use crate::entities::offer::NewOfferDto;
use crate::grpc::api::negotiation_agent::{
    CreateOfferRequest, GetBatchOffersRequest, ListOffersRequest, Offer as ProtoOffer,
    OfferListResponse, OfferResponse,
};
use crate::services::offer::views::OfferView;
use common::batch_requests::BatchRequests;
use common::grpc::{JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoField, ProtoFieldList};
use common::paginated_spec::Paginated;
use serde_json::Value as Json;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListOffersRequest> for ListParams<OfferFilter> {
    type Error = Status;

    fn try_from(req: ListOffersRequest) -> Result<Self, Status> {
        let filter = OfferFilter {
            id: None,
            tenant_id: None,
            process_id: req.process_id.non_empty().map(str::to_owned),
            offer_id: req.offer_id.non_empty().map(str::to_owned),
            target: req.target.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<GetBatchOffersRequest> for BatchRequests {
    type Error = Status;

    fn try_from(req: GetBatchOffersRequest) -> Result<Self, Status> {
        Ok(Self {
            ids: req.ids.urns("ids")?,
        })
    }
}

impl TryFrom<CreateOfferRequest> for NewOfferDto {
    type Error = Status;

    fn try_from(req: CreateOfferRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            negotiation_agent_process_id: req
                .negotiation_agent_process_id
                .urn("negotiation_agent_process_id")?,
            negotiation_agent_message_id: req
                .negotiation_agent_message_id
                .urn("negotiation_agent_message_id")?,
            offer_id: req.offer_id,
            offer_content: req
                .offer_content
                .map(JsonStructExt::into_json)
                .unwrap_or(Json::Object(Default::default())),
        })
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<OfferView> for ProtoOffer {
    fn from(view: OfferView) -> Self {
        let inner = view.inner;
        Self {
            id: inner.id,
            negotiation_process_id: inner.negotiation_agent_process_id,
            negotiation_message_id: inner.negotiation_agent_message_id,
            offer_id: inner.offer_id,
            offer_content: Some(inner.offer_content.into_prost_struct()),
            created_at: inner.created_at.to_rfc3339(),
        }
    }
}

impl From<OfferView> for OfferResponse {
    fn from(view: OfferView) -> Self {
        Self {
            offer: Some(view.into()),
        }
    }
}

impl From<Paginated<OfferView>> for OfferListResponse {
    fn from(p: Paginated<OfferView>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch / by-process results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<OfferView>> for OfferListResponse {
    fn from(views: Vec<OfferView>) -> Self {
        Self {
            total: views.len() as u64,
            items: views.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
