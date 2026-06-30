/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

mod mappers;

use std::str::FromStr;
use std::sync::Arc;

use common::auth::claims::{Claims, Role};
use common::auth::middleware::TokenValidator;
use common::auth::rbac::Rbac;
use tonic::{Request, Response, Status};
use urn::Urn;

use crate::entities::ids::TenantId;
use crate::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, DeleteResponse, ListTransferMessagesByProcessRequest,
    ListTransferMessagesRequest, ResourceIdRequest, TransferMessageListResponse,
    TransferMessageResponse, transfer_messages_ref_server::TransferMessagesRef,
};
use crate::services::transfer_message::TransferMessageServiceTrait;

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
    ) -> Result<(Claims, TenantId), Status> {
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

        Ok((claims, TenantId::new(tenant_raw)))
    }

    fn check_read(claims: &Claims, tenant_id: &TenantId) -> Result<(), Status> {
        Rbac::require_read(claims, tenant_id.as_str())
            .map_err(|e| Status::permission_denied(e.to_string()))
    }

    fn check_write(claims: &Claims, tenant_id: &TenantId) -> Result<(), Status> {
        Rbac::require_write(claims, tenant_id.as_str())
            .map_err(|e| Status::permission_denied(e.to_string()))
    }

    fn tenant_filter(claims: &Claims, tenant_id: &TenantId) -> Option<TenantId> {
        (claims.role != Role::Admin).then(|| tenant_id.clone())
    }
}

#[tonic::async_trait]
impl TransferMessagesRef for TransferMessagesGrpc {
    async fn list_transfer_messages(
        &self,
        request: Request<ListTransferMessagesRequest>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let (claims, tenant_id) = self.extract_auth(&meta).await?;
        Self::check_read(&claims, &tenant_id)?;
        let tenant_filter = Self::tenant_filter(&claims, &tenant_id);
        let (filter, page, sort) = mappers::into_list_params(proto_req, tenant_filter)?;
        let result = self
            .service
            .get_all(&filter, &page, &sort)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(mappers::from_paginated(result)))
    }

    async fn list_transfer_messages_by_process(
        &self,
        request: Request<ListTransferMessagesByProcessRequest>,
    ) -> Result<Response<TransferMessageListResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let (claims, tenant_id) = self.extract_auth(&meta).await?;
        Self::check_read(&claims, &tenant_id)?;
        let tenant_filter = Self::tenant_filter(&claims, &tenant_id);
        let (process_urn, filter, page, sort) =
            mappers::into_list_by_process_params(proto_req, tenant_filter)?;
        let result = self
            .service
            .get_all_by_process(&process_urn, &filter, &page, &sort)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(mappers::from_paginated(result)))
    }

    async fn get_transfer_message(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let (claims, tenant_id) = self.extract_auth(&meta).await?;
        Self::check_read(&claims, &tenant_id)?;
        let urn = parse_urn(&proto_req.id)?;
        let view = self
            .service
            .get_one(&urn)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        if claims.role != Role::Admin && view.tenant_id != tenant_id {
            return Err(Status::not_found("transfer message not found"));
        }
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn create_transfer_message(
        &self,
        request: Request<CreateTransferMessageRequest>,
    ) -> Result<Response<TransferMessageResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let (claims, tenant_id) = self.extract_auth(&meta).await?;
        Self::check_write(&claims, &tenant_id)?;
        let cmd = mappers::into_create_cmd(proto_req, tenant_id)?;
        let view = self
            .service
            .create(&cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn delete_transfer_message(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let (claims, tenant_id) = self.extract_auth(&meta).await?;
        Self::check_write(&claims, &tenant_id)?;
        let urn = parse_urn(&proto_req.id)?;
        // Verify tenant ownership before deleting; surfaces not-found for foreign records.
        if claims.role != Role::Admin {
            let existing = self
                .service
                .get_one(&urn)
                .await
                .map_err(|e| Status::not_found(e.to_string()))?;
            if existing.tenant_id != tenant_id {
                return Err(Status::not_found("transfer message not found"));
            }
        }
        self.service
            .delete(&urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteResponse {}))
    }
}

fn parse_urn(s: &str) -> Result<Urn, Status> {
    Urn::from_str(s).map_err(|e| Status::invalid_argument(format!("invalid URN: {e}")))
}
