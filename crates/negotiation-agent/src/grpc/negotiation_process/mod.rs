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

//! gRPC adapter for negotiation process management service.

mod mappers;

use std::sync::Arc;

use crate::grpc::api::negotiation_agent::negotiation_agent_processes_service_server::NegotiationAgentProcessesService;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationProcessRequest, DeleteNegotiationProcessRequest,
    GetBatchNegotiationProcessesRequest, GetNegotiationProcessByIdRequest,
    GetNegotiationProcessByKeyIdRequest, GetNegotiationProcessByKeyValueRequest,
    ListNegotiationProcessesRequest, NegotiationProcessListResponse, NegotiationProcessResponse,
    PutNegotiationProcessRequest,
};
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::grpc::GrpcAuth;
use common::batch_requests::BatchRequests;
use common::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct NegotiationAgentProcessesGrpc {
    service: Arc<dyn NegotiationProcessServiceTrait>,
    auth: GrpcAuth,
}

impl NegotiationAgentProcessesGrpc {
    pub fn new(
        service: Arc<dyn NegotiationProcessServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl NegotiationAgentProcessesService for NegotiationAgentProcessesGrpc {
    async fn get_all_negotiation_processes(
        &self,
        request: Request<ListNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_negotiation_processes(
        &self,
        request: Request<GetBatchNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let batch = BatchRequests::try_from(request.into_inner())?;
        let views = self
            .service
            .batch(&scope, &batch)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(views.into()))
    }

    async fn get_negotiation_process_by_id(
        &self,
        request: Request<GetNegotiationProcessByIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_one(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_negotiation_process_by_key_id(
        &self,
        request: Request<GetNegotiationProcessByKeyIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let view = self
            .service
            .get_by_key_id(&scope, &req.key_id, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_negotiation_process_by_key_value(
        &self,
        request: Request<GetNegotiationProcessByKeyValueRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_by_key_value(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_negotiation_process(
        &self,
        request: Request<CreateNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let view = self
            .service
            .create(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn put_negotiation_process(
        &self,
        request: Request<PutNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
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

    async fn delete_negotiation_process(
        &self,
        request: Request<DeleteNegotiationProcessRequest>,
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
