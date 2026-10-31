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

//! gRPC adapter for negotiation message management service.

mod mappers;

use std::sync::Arc;

use crate::grpc::api::negotiation_agent::negotiation_agent_messages_service_server::NegotiationAgentMessagesService;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationMessageRequest, DeleteNegotiationMessageRequest,
    GetMessagesByProcessIdRequest, GetNegotiationMessageByIdRequest,
    ListNegotiationMessagesRequest, NegotiationMessageListResponse, NegotiationMessageResponse,
};
use crate::setup::context::AppContext;
use ymir::oauth::grpc::GrpcAuth;
use ymir::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

/// gRPC adapter of the message service.
pub struct NegotiationAgentMessagesGrpc {
    ctx: Arc<AppContext>,
    auth: GrpcAuth,
}

impl NegotiationAgentMessagesGrpc {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            auth: GrpcAuth::new(ctx.oauth_validator.clone()),
            ctx,
        }
    }
}

#[tonic::async_trait]
impl NegotiationAgentMessagesService for NegotiationAgentMessagesGrpc {
    async fn get_all_negotiation_messages(
        &self,
        request: Request<ListNegotiationMessagesRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self.ctx.messages()
            .get_all(&user, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_messages_by_process_id(
        &self,
        request: Request<GetMessagesByProcessIdRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self.ctx.messages()
            .get_all(&user, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_negotiation_message_by_id(
        &self,
        request: Request<GetNegotiationMessageByIdRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self.ctx.messages()
            .get_one(&user, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_negotiation_message(
        &self,
        request: Request<CreateNegotiationMessageRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let view = self.ctx.messages()
            .create(&user, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_negotiation_message(
        &self,
        request: Request<DeleteNegotiationMessageRequest>,
    ) -> Result<Response<()>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.ctx.messages()
            .delete(&user, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
