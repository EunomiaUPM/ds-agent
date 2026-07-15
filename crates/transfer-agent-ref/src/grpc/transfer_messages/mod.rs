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

use std::str::FromStr;
use std::sync::Arc;

use crate::entities::ids::TenantId;
use crate::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, DeleteResponse, ListTransferMessagesByProcessRequest,
    ListTransferMessagesRequest, ResourceIdRequest, TransferMessageListResponse,
    TransferMessageResponse, transfer_messages_ref_server::TransferMessagesRef,
};
use crate::grpc::to_status;
use crate::services::transfer_message::TransferMessageServiceTrait;
use common::auth::access::AccessScope;
use common::auth::claims::Claims;
use common::auth::middleware::TokenValidator;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct TransferMessagesGrpc {
    service: Arc<dyn TransferMessageServiceTrait>,
    validator: Arc<dyn TokenValidator>,
}

impl TransferMessagesGrpc {
    pub fn new(
        service: Arc<dyn TransferMessageServiceTrait>,
        validator: Arc<dyn TokenValidator>,
    ) -> Self {
        Self { service, validator }
    }

    async fn extract_auth(
        &self,
        meta: &tonic::metadata::MetadataMap,
    ) -> Result<(Claims, String), Status> {
        let token = meta
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("missing Authorization metadata"))?;

        let claims = self
            .validator
            .validate_token(token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let tenant_raw = meta
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::invalid_argument("missing x-tenant-id metadata"))?;

        Ok((claims, tenant_raw.to_string()))
    }

    /// Builds the caller's read scope (RBAC + tenant) from request metadata.
    async fn read_scope(&self, meta: &tonic::metadata::MetadataMap) -> Result<AccessScope, Status> {
        let (claims, tenant) = self.extract_auth(meta).await?;
        AccessScope::for_read(&claims, &tenant).map_err(to_status)
    }

    /// Builds the caller's write scope (RBAC + tenant) from request metadata.
    async fn write_scope(
        &self,
        meta: &tonic::metadata::MetadataMap,
    ) -> Result<AccessScope, Status> {
        let (claims, tenant) = self.extract_auth(meta).await?;
        AccessScope::for_write(&claims, &tenant).map_err(to_status)
    }
}

#[tonic::async_trait]
impl TransferMessagesRef for TransferMessagesGrpc {
    async fn list_transfer_messages(
        &self,
        request: Request<ListTransferMessagesRequest>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.read_scope(&meta).await?;
        let (filter, page, sort) = mappers::into_list_params(proto_req)?;
        let result = self
            .service
            .get_all(&scope, &filter, &page, &sort)
            .await
            .map_err(to_status)?;
        Ok(Response::new(mappers::from_paginated(result)))
    }

    async fn list_transfer_messages_by_process(
        &self,
        request: Request<ListTransferMessagesByProcessRequest>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.read_scope(&meta).await?;
        let (process_urn, filter, page, sort) = mappers::into_list_by_process_params(proto_req)?;
        let result = self
            .service
            .get_all_by_process(&scope, &process_urn, &filter, &page, &sort)
            .await
            .map_err(to_status)?;
        Ok(Response::new(mappers::from_paginated(result)))
    }

    async fn get_transfer_message(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.read_scope(&meta).await?;
        let urn = parse_urn(&proto_req.id)?;
        let view = self
            .service
            .get_one(&scope, &urn)
            .await
            .map_err(to_status)?;
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn create_transfer_message(
        &self,
        request: Request<CreateTransferMessageRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.write_scope(&meta).await?;
        let cmd = mappers::into_create_cmd(proto_req)?;
        let view = self.service.create(&scope, &cmd).await.map_err(to_status)?;
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn delete_transfer_message(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.write_scope(&meta).await?;
        let urn = parse_urn(&proto_req.id)?;
        self.service.delete(&scope, &urn).await.map_err(to_status)?;
        Ok(Response::new(DeleteResponse {}))
    }
}

fn parse_urn(s: &str) -> Result<Urn, Status> {
    Urn::from_str(s).map_err(|e| Status::invalid_argument(format!("invalid URN: {e}")))
}
