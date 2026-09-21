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

//! gRPC adapter for offer management service.

use crate::entities::offer::{NewOfferDto, OfferDto};
use crate::grpc::api::negotiation_agent::negotiation_agent_offers_service_server::NegotiationAgentOffersService;
use crate::grpc::api::negotiation_agent::{
    CreateOfferRequest, DeleteOfferRequest, GetAllOffersRequest, GetBatchOffersRequest,
    GetOfferByIdRequest, GetOfferByNegotiationMessageRequest, GetOfferByOfferIdRequest,
    GetOffersByNegotiationProcessRequest, OfferListResponse, OfferResponse,
};
use crate::grpc::{GrpcAuthHelper, IntoGrpcStatus};
use crate::services::offer::OfferServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentOfferGrpc {
    service: Arc<dyn OfferServiceTrait>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl NegotiationAgentOfferGrpc {
    pub fn new(
        service: Arc<dyn OfferServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self { service, validator }
    }

    async fn scope(&self, meta: &tonic::metadata::MetadataMap) -> Result<AccessScope, Status> {
        GrpcAuthHelper::extract_scope(&self.validator, meta).await
    }
}

#[tonic::async_trait]
impl NegotiationAgentOffersService for NegotiationAgentOfferGrpc {
    async fn get_all_offers(
        &self,
        request: Request<GetAllOffersRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let paginated = self
            .service
            .get_all(&scope, &Default::default(), &page, &Default::default())
            .await
            .map_err(|e| e.into_status())?;

        let proto_offers = paginated
            .items
            .into_iter()
            .map(|view| {
                let dto: OfferDto = view.into();
                let response: OfferResponse = dto.into();
                response.offer.unwrap()
            })
            .collect();

        Ok(Response::new(OfferListResponse {
            offers: proto_offers,
        }))
    }

    async fn get_batch_offers(
        &self,
        request: Request<GetBatchOffersRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid URN in batch: {e}")))?;

        let batch_req = BatchRequests { ids: urns };
        let views = self
            .service
            .batch(&scope, &batch_req)
            .await
            .map_err(|e| e.into_status())?;

        let proto_offers = views
            .into_iter()
            .map(|view| {
                let dto: OfferDto = view.into();
                let response: OfferResponse = dto.into();
                response.offer.unwrap()
            })
            .collect();

        Ok(Response::new(OfferListResponse {
            offers: proto_offers,
        }))
    }

    async fn get_offers_by_negotiation_process(
        &self,
        request: Request<GetOffersByNegotiationProcessRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {e}")))?;

        let views = self
            .service
            .get_by_process(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;

        let proto_offers = views
            .into_iter()
            .map(|view| {
                let dto: OfferDto = view.into();
                let response: OfferResponse = dto.into();
                response.offer.unwrap()
            })
            .collect();

        Ok(Response::new(OfferListResponse {
            offers: proto_offers,
        }))
    }

    async fn get_offer_by_id(
        &self,
        request: Request<GetOfferByIdRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;

        let view = self
            .service
            .get_one(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: OfferDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn get_offer_by_negotiation_message(
        &self,
        request: Request<GetOfferByNegotiationMessageRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.message_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Message ID URN: {e}")))?;

        let view = self
            .service
            .get_by_negotiation_message(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: OfferDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn get_offer_by_offer_id(
        &self,
        request: Request<GetOfferByOfferIdRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.offer_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Offer ID URN: {e}")))?;

        let view = self
            .service
            .get_by_offer_id(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: OfferDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn create_offer(
        &self,
        request: Request<CreateOfferRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let new_offer_dto: NewOfferDto = req.try_into()?;

        let view = self
            .service
            .create(&scope, &new_offer_dto)
            .await
            .map_err(|e| e.into_status())?;
        let dto: OfferDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn delete_offer(
        &self,
        request: Request<DeleteOfferRequest>,
    ) -> Result<Response<()>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;

        self.service
            .delete(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        Ok(Response::new(()))
    }
}
