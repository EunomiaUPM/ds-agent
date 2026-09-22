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

mod mappers;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, DeleteResponse,
    EditTransferProcessRequest, ListTransferProcessesRequest, ResourceIdRequest,
    TransferProcessListResponse, TransferProcessResponse,
    transfer_processes_ref_server::TransferProcessesRef,
};
use crate::grpc::transfer_process::mappers::ListParams;
use crate::services::transfer_process::TransferProcessServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::grpc::GrpcAuth;
use common::batch_requests::BatchRequests;
use common::grpc::{IntoStatus, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct TransferProcessGrpc {
    service: Arc<dyn TransferProcessServiceTrait>,
    auth: GrpcAuth,
}

impl TransferProcessGrpc {
    pub fn new(
        service: Arc<dyn TransferProcessServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl TransferProcessesRef for TransferProcessGrpc {
    async fn list_transfer_processes(
        &self,
        request: Request<ListTransferProcessesRequest>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_transfer_process(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_one(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn batch_get_transfer_processes(
        &self,
        request: Request<BatchTransferProcessesRequest>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let batch = BatchRequests::try_from(request.into_inner())?;
        let views = self
            .service
            .batch(&scope, &batch)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(views.into()))
    }

    async fn create_transfer_process(
        &self,
        request: Request<CreateTransferProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let cmd = request.into_inner().try_into()?;
        let view = self
            .service
            .create(&scope, &cmd)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn edit_transfer_process(
        &self,
        request: Request<EditTransferProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let cmd = req.try_into()?;
        let view = self
            .service
            .edit(&scope, &id, &cmd)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_transfer_process(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(DeleteResponse {}))
    }
}
