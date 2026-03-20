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

use crate::entities::negotiation_message::{
    NegotiationAgentMessagesTrait, NewNegotiationMessageDto,
};
use crate::grpc::api::negotiation_agent::negotiation_agent_messages_service_server::NegotiationAgentMessagesService;
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationMessageRequest, DeleteNegotiationMessageRequest,
    GetAllNegotiationMessagesRequest, GetMessagesByProcessIdRequest,
    GetNegotiationMessageByIdRequest, NegotiationMessageListResponse, NegotiationMessageResponse,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentMessagesGrpc {
    service: Arc<dyn NegotiationAgentMessagesTrait>,
}

impl NegotiationAgentMessagesGrpc {
    pub fn new(service: Arc<dyn NegotiationAgentMessagesTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl NegotiationAgentMessagesService for NegotiationAgentMessagesGrpc {
    async fn get_all_negotiation_messages(
        &self,
        request: Request<GetAllNegotiationMessagesRequest>,
    ) -> Result<Response<NegotiationMessageListResponse>, Status> {
        let req = request.into_inner();

        let messages = self
            .service
            .get_all_negotiation_messages(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_messages = messages
            .into_iter()
            .map(|dto| {
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
        let req = request.into_inner();
        let urn = Urn::from_str(&req.process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {}", e)))?;

        let messages = self
            .service
            .get_messages_by_process_id(&urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_messages = messages
            .into_iter()
            .map(|dto| {
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
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.get_negotiation_message_by_id(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Negotiation message not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_negotiation_message(
        &self,
        request: Request<CreateNegotiationMessageRequest>,
    ) -> Result<Response<NegotiationMessageResponse>, Status> {
        let req = request.into_inner();

        let new_message_dto: NewNegotiationMessageDto = req.try_into()?;

        match self.service.create_negotiation_message(&new_message_dto).await {
            Ok(dto) => Ok(Response::new(dto.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_negotiation_message(
        &self,
        request: Request<DeleteNegotiationMessageRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.delete_negotiation_message(&urn).await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
