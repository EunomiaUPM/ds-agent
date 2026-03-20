/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateEntityTrait};
use crate::grpc::api::catalog_agent::policy_template_entity_service_server::PolicyTemplateEntityService;
use crate::grpc::api::catalog_agent::{
    CreatePolicyTemplateRequest, DeleteByIdRequest, DeleteByVersionRequest, GetAllRequest,
    GetBatchRequest, GetByIdRequest, GetByVersionRequest, PolicyTemplate,
    PolicyTemplateListResponse, PolicyTemplateResponse,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct PolicyTemplateEntityGrpc {
    service: Arc<dyn PolicyTemplateEntityTrait>,
}

impl PolicyTemplateEntityGrpc {
    pub fn new(service: Arc<dyn PolicyTemplateEntityTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl PolicyTemplateEntityService for PolicyTemplateEntityGrpc {
    async fn get_all_policy_templates(
        &self,
        request: Request<GetAllRequest>,
    ) -> Result<Response<PolicyTemplateListResponse>, Status> {
        let req = request.into_inner();
        let templates = self
            .service
            .get_all_policy_templates(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_templates: Vec<PolicyTemplate> = templates.into_iter().map(Into::into).collect();

        Ok(Response::new(PolicyTemplateListResponse {
            policy_templates: proto_templates,
        }))
    }

    async fn get_batch_policy_templates(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<PolicyTemplateListResponse>, Status> {
        let req = request.into_inner();
        let urns = req.ids;

        let templates = self
            .service
            .get_batch_policy_templates(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_templates: Vec<PolicyTemplate> = templates.into_iter().map(Into::into).collect();

        Ok(Response::new(PolicyTemplateListResponse {
            policy_templates: proto_templates,
        }))
    }

    async fn get_policy_templates_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<PolicyTemplateListResponse>, Status> {
        let req = request.into_inner();
        let policy_template = req.id;

        let templates = self
            .service
            .get_policies_template_by_id(&policy_template)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let proto_templates: Vec<PolicyTemplate> = templates.into_iter().map(Into::into).collect();

        Ok(Response::new(PolicyTemplateListResponse {
            policy_templates: proto_templates,
        }))
    }

    async fn get_policy_template_by_version(
        &self,
        request: Request<GetByVersionRequest>,
    ) -> Result<Response<PolicyTemplateResponse>, Status> {
        let req = request.into_inner();
        let policy_template = req.id;
        let version = req.version;
        let templates = self
            .service
            .get_policies_template_by_version_and_id(&policy_template, &version)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        match templates {
            Some(dto) => Ok(Response::new(PolicyTemplateResponse {
                policy_template: Some(dto.into()),
            })),
            None => Err(Status::not_found("Policy template not found")),
        }
    }

    async fn create_policy_template(
        &self,
        request: Request<CreatePolicyTemplateRequest>,
    ) -> Result<Response<PolicyTemplateResponse>, Status> {
        let req = request.into_inner();
        let new_template_dto: NewPolicyTemplateDto = req.try_into()?;

        let created_dto =
            self.service.create_policy_template(&new_template_dto).await.map_err(|e| {
                Status::internal(format!("Failed to create Policy Template: {}", e))
            })?;

        Ok(Response::new(PolicyTemplateResponse {
            policy_template: Some(created_dto.into()),
        }))
    }

    async fn delete_policy_template_by_version(
        &self,
        request: Request<DeleteByVersionRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let policy_template = req.id;
        let version = req.version;

        let _ = self
            .service
            .delete_policy_template_by_version_and_id(&policy_template, &version)
            .await
            .map_err(|e| Status::internal(format!("Failed to create Policy Template: {}", e)))?;

        Ok(Response::new(()))
    }
}
