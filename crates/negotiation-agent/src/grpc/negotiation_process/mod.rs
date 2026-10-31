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
use crate::setup::context::AppContext;
use ymir::oauth::grpc::GrpcAuth;
use ymir::batch_requests::BatchRequests;
use ymir::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

/// gRPC adapter of the process service.
pub struct NegotiationAgentProcessesGrpc {
    ctx: Arc<AppContext>,
    auth: GrpcAuth,
}

impl NegotiationAgentProcessesGrpc {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            auth: GrpcAuth::new(ctx.oauth_validator.clone()),
            ctx,
        }
    }
}

#[tonic::async_trait]
impl NegotiationAgentProcessesService for NegotiationAgentProcessesGrpc {
    async fn get_all_negotiation_processes(
        &self,
        request: Request<ListNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self.ctx.processes()
            .get_all(&user, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_negotiation_processes(
        &self,
        request: Request<GetBatchNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let batch = BatchRequests::try_from(request.into_inner())?;
        let views = self.ctx.processes()
            .batch(&user, &batch)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(views.into()))
    }

    async fn get_negotiation_process_by_id(
        &self,
        request: Request<GetNegotiationProcessByIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self.ctx.processes()
            .get_one(&user, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_negotiation_process_by_key_id(
        &self,
        request: Request<GetNegotiationProcessByKeyIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let view = self.ctx.processes()
            .get_by_key_id(&user, &req.key_id, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn get_negotiation_process_by_key_value(
        &self,
        request: Request<GetNegotiationProcessByKeyValueRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id;
        let view = self.ctx.processes()
            .get_by_key_value(&user, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_negotiation_process(
        &self,
        request: Request<CreateNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let view = self.ctx.processes()
            .create(&user, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn put_negotiation_process(
        &self,
        request: Request<PutNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let view = self.ctx.processes()
            .edit(&user, &id, &req.into())
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_negotiation_process(
        &self,
        request: Request<DeleteNegotiationProcessRequest>,
    ) -> Result<Response<()>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.ctx.processes()
            .delete(&user, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
