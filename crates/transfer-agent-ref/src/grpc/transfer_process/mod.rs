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

use common::auth::claims::Claims;
use common::auth::middleware::TokenValidator;
use tonic::{Request, Response, Status};
use urn::Urn;

use crate::entities::ids::TenantId;
use crate::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, DeleteResponse,
    EditTransferProcessRequest, ListTransferProcessesRequest, ResourceIdRequest,
    TransferProcessListResponse, TransferProcessResponse,
    transfer_processes_ref_server::TransferProcessesRef,
};
use crate::grpc::to_status;
use crate::services::access::AccessScope;
use crate::services::transfer_process::TransferProcessServiceTrait;

pub struct TransferProcessGrpc {
    service: Arc<dyn TransferProcessServiceTrait>,
    validator: Arc<dyn TokenValidator>,
}

impl TransferProcessGrpc {
    pub fn new(
        service: Arc<dyn TransferProcessServiceTrait>,
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
impl TransferProcessesRef for TransferProcessGrpc {
    async fn list_transfer_processes(
        &self,
        request: Request<ListTransferProcessesRequest>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
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

    async fn get_transfer_process(
        &self,
        request: Request<ResourceIdRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
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

    async fn batch_get_transfer_processes(
        &self,
        request: Request<BatchTransferProcessesRequest>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.read_scope(&meta).await?;
        let batch = mappers::into_batch(proto_req)?;
        let views = self
            .service
            .batch(&scope, &batch)
            .await
            .map_err(to_status)?;
        Ok(Response::new(mappers::from_vec(views)))
    }

    async fn create_transfer_process(
        &self,
        request: Request<CreateTransferProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.write_scope(&meta).await?;
        let cmd = mappers::into_create_cmd(proto_req)?;
        let view = self.service.create(&scope, &cmd).await.map_err(to_status)?;
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn edit_transfer_process(
        &self,
        request: Request<EditTransferProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.write_scope(&meta).await?;
        let urn = parse_urn(&proto_req.id)?;
        let cmd = mappers::into_edit_cmd(proto_req)?;
        let view = self
            .service
            .edit(&scope, &urn, &cmd)
            .await
            .map_err(to_status)?;
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn delete_transfer_process(
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
