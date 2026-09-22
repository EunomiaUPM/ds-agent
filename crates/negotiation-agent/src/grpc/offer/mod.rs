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

mod mappers;

use std::sync::Arc;

use crate::grpc::api::negotiation_agent::negotiation_agent_offers_service_server::NegotiationAgentOffersService;
use crate::grpc::api::negotiation_agent::{
    CreateOfferRequest, DeleteOfferRequest, GetBatchOffersRequest, GetOfferByIdRequest,
    GetOfferByNegotiationMessageRequest, GetOfferByOfferIdRequest,
    GetOffersByNegotiationProcessRequest, ListOffersRequest, OfferListResponse, OfferResponse,
};
use crate::services::offer::OfferServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::grpc::GrpcAuth;
use common::batch_requests::BatchRequests;
use common::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct NegotiationAgentOfferGrpc {
    service: Arc<dyn OfferServiceTrait>,
    auth: GrpcAuth,
}

impl NegotiationAgentOfferGrpc {
    pub fn new(
        service: Arc<dyn OfferServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl NegotiationAgentOffersService for NegotiationAgentOfferGrpc {
    async fn get_all_offers(
        &self,
        request: Request<ListOffersRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_offers(
        &self,
        request: Request<GetBatchOffersRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let batch = BatchRequests::try_from(request.into_inner())?;
        let views = self
            .service
            .batch(&scope, &batch)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(views.into()))
    }

    async fn get_offers_by_negotiation_process(
        &self,
        request: Request<GetOffersByNegotiationProcessRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let process_id = request.into_inner().process_id.urn("process_id")?;
        let views = self
            .service
            .get_by_process(&scope, &process_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(views.into()))
    }

    async fn get_offer_by_id(
        &self,
        request: Request<GetOfferByIdRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_one(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_offer_by_negotiation_message(
        &self,
        request: Request<GetOfferByNegotiationMessageRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let message_id = request.into_inner().message_id.urn("message_id")?;
        let view = self
            .service
            .get_by_negotiation_message(&scope, &message_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_offer_by_offer_id(
        &self,
        request: Request<GetOfferByOfferIdRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let offer_id = request.into_inner().offer_id.urn("offer_id")?;
        let view = self
            .service
            .get_by_offer_id(&scope, &offer_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_offer(
        &self,
        request: Request<CreateOfferRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let view = self
            .service
            .create(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_offer(
        &self,
        request: Request<DeleteOfferRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
