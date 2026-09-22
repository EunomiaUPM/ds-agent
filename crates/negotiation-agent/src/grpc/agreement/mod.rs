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

//! gRPC adapter for agreement management service.

mod mappers;

use std::sync::Arc;

use crate::grpc::api::negotiation_agent::negotiation_agent_agreements_service_server::NegotiationAgentAgreementsService;
use crate::grpc::api::negotiation_agent::{
    AgreementListResponse, AgreementResponse, CreateAgreementRequest, DeleteAgreementRequest,
    GetAgreementByIdRequest, GetAgreementByNegotiationMessageRequest,
    GetAgreementByNegotiationProcessRequest, GetBatchAgreementsRequest, ListAgreementsRequest,
    PutAgreementRequest,
};
use crate::services::agreement::AgreementServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::grpc::GrpcAuth;
use common::batch_requests::BatchRequests;
use common::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct NegotiationAgentAgreementGrpc {
    service: Arc<dyn AgreementServiceTrait>,
    auth: GrpcAuth,
}

impl NegotiationAgentAgreementGrpc {
    pub fn new(
        service: Arc<dyn AgreementServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl NegotiationAgentAgreementsService for NegotiationAgentAgreementGrpc {
    async fn get_all_agreements(
        &self,
        request: Request<ListAgreementsRequest>,
    ) -> Result<Response<AgreementListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_agreements(
        &self,
        request: Request<GetBatchAgreementsRequest>,
    ) -> Result<Response<AgreementListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let batch = BatchRequests::try_from(request.into_inner())?;
        let views = self
            .service
            .batch(&scope, &batch)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(views.into()))
    }

    async fn get_agreement_by_id(
        &self,
        request: Request<GetAgreementByIdRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_one(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        request: Request<GetAgreementByNegotiationProcessRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let process_id = request.into_inner().process_id.urn("process_id")?;
        let view = self
            .service
            .get_by_process(&scope, &process_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        request: Request<GetAgreementByNegotiationMessageRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let message_id = request.into_inner().message_id.urn("message_id")?;
        let view = self
            .service
            .get_by_message(&scope, &message_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_agreement(
        &self,
        request: Request<CreateAgreementRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let view = self
            .service
            .create(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn put_agreement(
        &self,
        request: Request<PutAgreementRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let view = self
            .service
            .edit(&scope, &id, &req.into())
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_agreement(
        &self,
        request: Request<DeleteAgreementRequest>,
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
