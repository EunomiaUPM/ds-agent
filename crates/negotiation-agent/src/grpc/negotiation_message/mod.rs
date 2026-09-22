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
use crate::services::negotiation_message::NegotiationMessageServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::grpc::GrpcAuth;
use common::grpc::{IntoStatus, ListParams, ProtoField};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct NegotiationAgentMessagesGrpc {
    service: Arc<dyn NegotiationMessageServiceTrait>,
    auth: GrpcAuth,
}

impl NegotiationAgentMessagesGrpc {
    pub fn new(
        service: Arc<dyn NegotiationMessageServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl NegotiationAgentMessagesService for NegotiationAgentMessagesGrpc {
    async fn get_all_negotiation_messages(
        &self,
        request: Request<ListNegotiationMessagesRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_messages_by_process_id(
        &self,
        request: Request<GetMessagesByProcessIdRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_negotiation_message_by_id(
        &self,
        request: Request<GetNegotiationMessageByIdRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let view = self
            .service
            .get_one(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn create_negotiation_message(
        &self,
        request: Request<CreateNegotiationMessageRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let view = self
            .service
            .create(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(view.into()))
    }

    async fn delete_negotiation_message(
        &self,
        request: Request<DeleteNegotiationMessageRequest>,
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
