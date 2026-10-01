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

use std::sync::Arc;

use crate::grpc::api::catalog_agent::policy_template_entity_service_server::PolicyTemplateEntityService;
use crate::grpc::api::catalog_agent::{
    CreatePolicyTemplateRequest, DeleteByVersionRequest, GetBatchRequest, GetByIdRequest,
    GetByVersionRequest, ListPolicyTemplatesRequest, PolicyTemplateListResponse,
    PolicyTemplateResponse,
};
use crate::services::policy_templates::PolicyTemplateServiceTrait;
use common::auth::grpc::GrpcAuth;
use common::auth::OauthTokenValidator;
use common::grpc::{IntoStatus, ListParams};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct PolicyTemplateEntityGrpc {
    service: Arc<dyn PolicyTemplateServiceTrait>,
    auth: GrpcAuth,
}

impl PolicyTemplateEntityGrpc {
    pub fn new(
        service: Arc<dyn PolicyTemplateServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl PolicyTemplateEntityService for PolicyTemplateEntityGrpc {
    async fn get_all_policy_templates(
        &self,
        request: Request<ListPolicyTemplatesRequest>,
    ) -> Result<Response<PolicyTemplateListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all_policy_templates(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.try_into()?))
    }

    async fn get_batch_policy_templates(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<PolicyTemplateListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ids = request.into_inner().ids;
        let dtos = self
            .service
            .get_batch_policy_templates(&scope, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.try_into()?))
    }

    async fn get_policy_templates_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<PolicyTemplateListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id;
        let dtos = self
            .service
            .get_policies_template_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.try_into()?))
    }

    async fn get_policy_template_by_version(
        &self,
        request: Request<GetByVersionRequest>,
    ) -> Result<Response<PolicyTemplateResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let dto = self
            .service
            .get_policies_template_by_version_and_id(&scope, &req.id, &req.version)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.try_into()?))
    }

    async fn create_policy_template(
        &self,
        request: Request<CreatePolicyTemplateRequest>,
    ) -> Result<Response<PolicyTemplateResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_policy_template(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.try_into()?))
    }

    async fn delete_policy_template_by_version(
        &self,
        request: Request<DeleteByVersionRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        self.service
            .delete_policy_template_by_version_and_id(&scope, &req.id, &req.version)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
