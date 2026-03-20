/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::entities::negotiation_process::{
    EditNegotiationProcessDto, NegotiationAgentProcessesTrait, NewNegotiationProcessDto,
};
use crate::grpc::api::negotiation_agent::negotiation_agent_processes_service_server::NegotiationAgentProcessesService;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationProcessRequest, DeleteNegotiationProcessRequest,
    GetAllNegotiationProcessesRequest, GetBatchNegotiationProcessesRequest,
    GetNegotiationProcessByIdRequest, GetNegotiationProcessByKeyIdRequest,
    GetNegotiationProcessByKeyValueRequest, NegotiationProcessListResponse,
    NegotiationProcessResponse, PutNegotiationProcessRequest,
};

use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentProcessesGrpc {
    service: Arc<dyn NegotiationAgentProcessesTrait>,
}

impl NegotiationAgentProcessesGrpc {
    pub fn new(service: Arc<dyn NegotiationAgentProcessesTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl NegotiationAgentProcessesService for NegotiationAgentProcessesGrpc {
    async fn get_all_negotiation_processes(
        &self,
        request: Request<GetAllNegotiationProcessesRequest>,
    ) -> Result<Response<NegotiationProcessListResponse>, Status> {
        let req = request.into_inner();

        let processes = self
            .service
            .get_all_negotiation_processes(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Convertimos DTOs a Protos.
        // Usamos la conversión Dto -> Response definida en mappers y extraemos el inner process.
        let proto_processes = processes
            .into_iter()
            .map(|dto| {
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
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid URN in batch: {}", e)))?;

        let processes = self
            .service
            .get_batch_negotiation_processes(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_processes = processes
            .into_iter()
            .map(|dto| {
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
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.get_negotiation_process_by_id(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Negotiation process not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_negotiation_process_by_key_id(
        &self,
        request: Request<GetNegotiationProcessByKeyIdRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {}", e)))?;

        match self.service.get_negotiation_process_by_key_id(&req.key_id, &urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Negotiation process not found by identifier key")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_negotiation_process_by_key_value(
        &self,
        request: Request<GetNegotiationProcessByKeyValueRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let req = request.into_inner();
        // Aquí 'id' representa el valor del URN del identificador
        let urn = Urn::from_str(&req.id).map_err(|e| {
            Status::invalid_argument(format!("Invalid Identifier Value URN: {}", e))
        })?;

        match self.service.get_negotiation_process_by_key_value(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Negotiation process not found by identifier value")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_negotiation_process(
        &self,
        request: Request<CreateNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let req = request.into_inner();

        // Conversión Request -> NewDto
        let new_process_dto: NewNegotiationProcessDto = req.try_into()?;

        match self.service.create_negotiation_process(&new_process_dto).await {
            Ok(dto) => Ok(Response::new(dto.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn put_negotiation_process(
        &self,
        request: Request<PutNegotiationProcessRequest>,
    ) -> Result<Response<NegotiationProcessResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        // Conversión Request -> EditDto
        let edit_process_dto: EditNegotiationProcessDto = req.try_into()?;

        match self.service.put_negotiation_process(&urn, &edit_process_dto).await {
            Ok(dto) => Ok(Response::new(dto.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_negotiation_process(
        &self,
        request: Request<DeleteNegotiationProcessRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.delete_negotiation_process(&urn).await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
