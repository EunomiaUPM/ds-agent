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
use crate::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, DeleteResponse,
    EditTransferProcessRequest, ListTransferProcessesRequest, ResourceIdRequest,
    TransferProcessListResponse, TransferProcessResponse,
    transfer_processes_ref_server::TransferProcessesRef,
};
use crate::grpc::to_status;
use crate::services::transfer_process::TransferProcessServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::access::AccessScope;
use common::auth::claims::Claims;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct TransferProcessGrpc {
    service: Arc<dyn TransferProcessServiceTrait>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl TransferProcessGrpc {
    pub fn new(
        service: Arc<dyn TransferProcessServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
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

        common::auth::validators::AuthValidators::claims_validator()
            .validate(&claims)
            .map_err(|vs| Status::unauthenticated(vs.to_string()))?;

        let tenant_raw = meta
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::invalid_argument("missing x-tenant-id metadata"))?;

        let tenant_id = tenant_raw.to_string();
        common::auth::validators::AuthValidators::tenant_id_validator()
            .validate(&tenant_id)
            .map_err(|vs| Status::invalid_argument(vs.to_string()))?;

        if !claims.is_admin() && claims.tenant_id() != tenant_id {
            return Err(Status::permission_denied(
                "forbidden: caller tenant does not match requested tenant",
            ));
        }

        Ok((claims, tenant_id))
    }

    /// Builds caller's access scope from validated metadata.
    async fn scope(&self, meta: &tonic::metadata::MetadataMap) -> Result<AccessScope, Status> {
        let (claims, tenant) = self.extract_auth(meta).await?;
        Ok(AccessScope::new(&claims, &tenant))
    }

    fn parse_urn(s: &str) -> Result<Urn, Status> {
        Urn::from_str(s).map_err(|e| Status::invalid_argument(format!("invalid URN: {e}")))
    }
}

#[tonic::async_trait]
impl TransferProcessesRef for TransferProcessGrpc {
    async fn list_transfer_processes(
        &self,
        request: Request<ListTransferProcessesRequest>,
    ) -> Result<Response<TransferProcessListResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.scope(&meta).await?;
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
        let scope = self.scope(&meta).await?;
        let urn = Self::parse_urn(&proto_req.id)?;
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
        let scope = self.scope(&meta).await?;
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
        let scope = self.scope(&meta).await?;
        let cmd = mappers::into_create_cmd(proto_req)?;
        let view = self.service.create(&scope, &cmd).await.map_err(to_status)?;
        Ok(Response::new(mappers::from_view(view)))
    }

    async fn edit_transfer_process(
        &self,
        request: Request<EditTransferProcessRequest>,
    ) -> Result<Response<TransferProcessResponse>, Status> {
        let (meta, _, proto_req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Self::parse_urn(&proto_req.id)?;
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
        let scope = self.scope(&meta).await?;
        let urn = Self::parse_urn(&proto_req.id)?;
        self.service.delete(&scope, &urn).await.map_err(to_status)?;
        Ok(Response::new(DeleteResponse {}))
    }
}
