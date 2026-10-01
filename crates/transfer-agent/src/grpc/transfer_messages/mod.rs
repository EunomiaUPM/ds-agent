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

use crate::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, DeleteResponse, ListTransferMessagesByProcessRequest,
    ListTransferMessagesRequest, ResourceIdRequest, TransferMessageListResponse,
    TransferMessageResponse, transfer_messages_ref_server::TransferMessagesRef,
};
use crate::grpc::transfer_messages::mappers::ListByProcessParams;
use crate::services::transfer_message::TransferMessageServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::grpc::GrpcAuth;
use common::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct TransferMessagesGrpc {
    service: Arc<dyn TransferMessageServiceTrait>,
    auth: GrpcAuth,
}

impl TransferMessagesGrpc {
    pub fn new(
        service: Arc<dyn TransferMessageServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl TransferMessagesRef for TransferMessagesGrpc {
    async fn list_transfer_messages(
        &self,
        request: Request<ListTransferMessagesRequest>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn list_transfer_messages_by_process(
        &self,
        request: Request<ListTransferMessagesByProcessRequest>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ListByProcessParams { process_id, params } = request.into_inner().try_into()?;
        let result = self
            .service
            .get_all_by_process(
                &scope,
                &process_id,
                &params.filter,
                &params.page,
                &params.sort,
            )
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_transfer_message(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_one(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_transfer_message(
        &self,
        request: Request<CreateTransferMessageRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let cmd = request.into_inner().try_into()?;
        let view = self
            .service
            .create(&scope, &cmd)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_transfer_message(
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
