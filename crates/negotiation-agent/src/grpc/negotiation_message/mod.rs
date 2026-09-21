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

use crate::entities::filters::NegotiationMessageFilter;
use crate::entities::negotiation_message::{NegotiationMessageDto, NewNegotiationMessageDto};
use crate::grpc::api::negotiation_agent::negotiation_agent_messages_service_server::NegotiationAgentMessagesService;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationMessageRequest, DeleteNegotiationMessageRequest,
    GetAllNegotiationMessagesRequest, GetMessagesByProcessIdRequest,
    GetNegotiationMessageByIdRequest, NegotiationMessageListResponse, NegotiationMessageResponse,
};
use crate::grpc::{GrpcAuthHelper, IntoGrpcStatus};
use crate::services::negotiation_message::NegotiationMessageServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::access::AccessScope;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentMessagesGrpc {
    service: Arc<dyn NegotiationMessageServiceTrait>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl NegotiationAgentMessagesGrpc {
    pub fn new(
        service: Arc<dyn NegotiationMessageServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self { service, validator }
    }

    async fn scope(&self, meta: &tonic::metadata::MetadataMap) -> Result<AccessScope, Status> {
        GrpcAuthHelper::extract_scope(&self.validator, meta).await
    }
}

#[tonic::async_trait]
impl NegotiationAgentMessagesService for NegotiationAgentMessagesGrpc {
    async fn get_all_negotiation_messages(
        &self,
        request: Request<GetAllNegotiationMessagesRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let paginated = self
            .service
            .get_all(&scope, &Default::default(), &page, &Default::default())
            .await
            .map_err(|e| e.into_status())?;

        let proto_messages = paginated
            .items
            .into_iter()
            .map(|view| {
                let dto: NegotiationMessageDto = view.into();
                let response: NegotiationMessageResponse = dto.into();
                response.message.unwrap()
            })
            .collect();

        Ok(Response::new(NegotiationMessageListResponse {
            messages: proto_messages,
        }))
    }

    async fn get_messages_by_process_id(
        &self,
        request: Request<GetMessagesByProcessIdRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {e}")))?;

        let filter = NegotiationMessageFilter {
            process_id: Some(urn.to_string()),
            ..Default::default()
        };
        let page = Page::new(100, None);
        let paginated = self
            .service
            .get_all(&scope, &filter, &page, &Default::default())
            .await
            .map_err(|e| e.into_status())?;

        let proto_messages = paginated
            .items
            .into_iter()
            .map(|view| {
                let dto: NegotiationMessageDto = view.into();
                let response: NegotiationMessageResponse = dto.into();
                response.message.unwrap()
            })
            .collect();

        Ok(Response::new(NegotiationMessageListResponse {
            messages: proto_messages,
        }))
    }

    async fn get_negotiation_message_by_id(
        &self,
        request: Request<GetNegotiationMessageByIdRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;

        let view = self
            .service
            .get_one(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationMessageDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn create_negotiation_message(
        &self,
        request: Request<CreateNegotiationMessageRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let new_message_dto: NewNegotiationMessageDto = req.try_into()?;

        let view = self
            .service
            .create(&scope, &new_message_dto)
            .await
            .map_err(|e| e.into_status())?;
        let dto: NegotiationMessageDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn delete_negotiation_message(
        &self,
        request: Request<DeleteNegotiationMessageRequest>,
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
