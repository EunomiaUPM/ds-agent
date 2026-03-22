/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

mod mappers;

use crate::entities::transfer_process::{NewTransferProcessDto, TransferAgentProcessesTrait};
use crate::grpc::api::transfer_processes::transfer_agent_processes_server::TransferAgentProcesses;
use crate::grpc::api::transfer_processes::{
    BatchProcessRequest, CreateProcessRequest, GetByKeyRequest, PaginationRequestProcesses,
    ResourceIdRequestProcesses, TransferProcessListResponse, TransferProcessResponse,
    UpdateProcessRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct TransferAgentProcessesGrpc {
    service: Arc<dyn TransferAgentProcessesTrait>,
}

impl TransferAgentProcessesGrpc {
    pub fn new(service: Arc<dyn TransferAgentProcessesTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl TransferAgentProcesses for TransferAgentProcessesGrpc {
    async fn get_all_processes(
        &self,
        request: Request<PaginationRequestProcesses>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
        let proto_req = request.into_inner();
        let params: PaginationRequestProcesses = proto_req.into();
        let processes = self
            .service
            .get_all_transfer_processes(params.limit, params.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let proto_processes = processes
            .into_iter()
            .map(|m| m.into()) // Llama a From<TransferMessageDto>
            .collect();
        Ok(Response::new(TransferProcessListResponse {
            processes: proto_processes,
        }))
    }

    async fn create_process(
        &self,
        request: Request<CreateProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let proto_req = request.into_inner();
        let request: CreateProcessRequest = proto_req.into();
        let new_transfer_process = NewTransferProcessDto::try_from(request)
            .map_err(|e| Status::internal(e.to_string()))?;
        let process = self
            .service
            .create_transfer_process(&new_transfer_process)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let proto_process: TransferProcessResponse = process.into();
        Ok(Response::new(proto_process))
    }

    async fn get_batch_processes(
        &self,
        _request: Request<BatchProcessRequest>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
        todo!()
    }

    async fn get_process_by_id(
        &self,
        _request: Request<ResourceIdRequestProcesses>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        todo!()
    }

    async fn update_process(
        &self,
        _request: Request<UpdateProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        todo!()
    }

    async fn delete_process(
        &self,
        _request: Request<ResourceIdRequestProcesses>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_process_by_key_id(
        &self,
        _request: Request<GetByKeyRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        todo!()
    }
}
