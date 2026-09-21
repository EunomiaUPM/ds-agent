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

use crate::entities::negotiation_process::{
    EditNegotiationProcessDto, NegotiationProcessDto, NewNegotiationProcessDto,
};
use crate::grpc::api::negotiation_agent::negotiation_agent_processes_service_server::NegotiationAgentProcessesService;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationProcessRequest, DeleteNegotiationProcessRequest,
    GetAllNegotiationProcessesRequest, GetBatchNegotiationProcessesRequest,
    GetNegotiationProcessByIdRequest, GetNegotiationProcessByKeyIdRequest,
    GetNegotiationProcessByKeyValueRequest, NegotiationProcessListResponse,
    NegotiationProcessResponse, PutNegotiationProcessRequest,
};
use crate::grpc::{GrpcAuthHelper, IntoGrpcStatus};
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentProcessesGrpc {
    service: Arc<dyn NegotiationProcessServiceTrait>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl NegotiationAgentProcessesGrpc {
    pub fn new(
        service: Arc<dyn NegotiationProcessServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self { service, validator }
    }

    async fn scope(&self, meta: &tonic::metadata::MetadataMap) -> Result<AccessScope, Status> {
        GrpcAuthHelper::extract_scope(&self.validator, meta).await
    }
}

#[tonic::async_trait]
impl NegotiationAgentProcessesService for NegotiationAgentProcessesGrpc {
    async fn get_all_negotiation_processes(
        &self,
        request: Request<GetAllNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let paginated = self
            .service
            .get_all(&scope, &Default::default(), &page, &Default::default())
            .await
            .map_err(|e| e.into_status())?;

        let proto_processes = paginated
            .items
            .into_iter()
            .map(|view| {
                let dto: NegotiationProcessDto = view.into();
                let response: NegotiationProcessResponse = dto.into();
                response.process.unwrap()
            })
            .collect();

        Ok(Response::new(NegotiationProcessListResponse {
            processes: proto_processes,
        }))
    }

    async fn get_batch_negotiation_processes(
        &self,
        request: Request<GetBatchNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
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

        let proto_processes = views
            .into_iter()
            .map(|view| {
                let dto: NegotiationProcessDto = view.into();
                let response: NegotiationProcessResponse = dto.into();
                response.process.unwrap()
            })
            .collect();

        Ok(Response::new(NegotiationProcessListResponse {
            processes: proto_processes,
        }))
    }

    async fn get_negotiation_process_by_id(
        &self,
        request: Request<GetNegotiationProcessByIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;

        let view = self
            .service
            .get_one(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationProcessDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn get_negotiation_process_by_key_id(
        &self,
        request: Request<GetNegotiationProcessByKeyIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {e}")))?;

        let view = self
            .service
            .get_by_key_id(&scope, &req.key_id, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationProcessDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn get_negotiation_process_by_key_value(
        &self,
        request: Request<GetNegotiationProcessByKeyValueRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Identifier Value URN: {e}")))?;

        let view = self
            .service
            .get_by_key_value(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationProcessDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn create_negotiation_process(
        &self,
        request: Request<CreateNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let new_process_dto: NewNegotiationProcessDto = req.try_into()?;

        let view = self
            .service
            .create(&scope, &new_process_dto)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationProcessDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn put_negotiation_process(
        &self,
        request: Request<PutNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;
        let edit_process_dto: EditNegotiationProcessDto = req.try_into()?;

        let view = self
            .service
            .edit(&scope, &urn, &edit_process_dto)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationProcessDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn delete_negotiation_process(
        &self,
        request: Request<DeleteNegotiationProcessRequest>,
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
