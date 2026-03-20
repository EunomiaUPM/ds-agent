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

use crate::entities::transfer_messages::TransferAgentMessagesTrait;
use crate::grpc::api::transfer_messages::transfer_agent_messages_server::TransferAgentMessages;
use crate::grpc::api::transfer_messages::{
    CreateMessageRequest, PaginationRequestMessages, ResourceIdRequestMessages,
    TransferMessageListResponse, TransferMessageResponse,
};
use crate::http::transfer_messages::PaginationParams;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct TransferAgentMessagesGrpc {
    service: Arc<dyn TransferAgentMessagesTrait>,
}

impl TransferAgentMessagesGrpc {
    pub fn new(service: Arc<dyn TransferAgentMessagesTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl TransferAgentMessages for TransferAgentMessagesGrpc {
    async fn get_all_messages(
        &self,
        request: Request<PaginationRequestMessages>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let proto_req = request.into_inner();
        let params: PaginationParams = proto_req.into();
        let messages = self
            .service
            .get_all_transfer_messages(params.limit, params.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let proto_messages = messages
            .into_iter()
            .map(|m| m.into()) // Llama a From<TransferMessageDto>
            .collect();
        Ok(Response::new(TransferMessageListResponse {
            messages: proto_messages,
        }))
    }

    async fn create_message(
        &self,
        _request: Request<CreateMessageRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        todo!()
    }

    async fn get_message_by_id(
        &self,
        _request: Request<ResourceIdRequestMessages>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        todo!()
    }

    async fn delete_message(
        &self,
        _request: Request<ResourceIdRequestMessages>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_messages_by_process_id(
        &self,
        _request: Request<ResourceIdRequestMessages>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        todo!()
    }
}
