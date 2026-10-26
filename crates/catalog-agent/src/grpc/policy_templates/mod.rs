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

//! The template RPCs over the template and instantiation services.

use std::sync::Arc;

use common::grpc::{IntoStatus, PageParams};
use common::oauth::grpc::GrpcAuth;
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_instantiation::NewPolicyInstantiationDto;
use crate::entities::policy_templates::NewPolicyTemplateDto;
use crate::grpc::api::catalog_agent::policy_template_service_server::PolicyTemplateService;
use crate::grpc::api::catalog_agent::{IdRequest, JsonDocument, ListJsonRequest, TemplateVersionRequest};
use crate::grpc::GrpcJson;
use crate::services::odrl_policies::views::OdrlPolicyView;
use crate::setup::context::AppContext;

pub struct PolicyTemplateGrpc {
    ctx: Arc<AppContext>,
    auth: GrpcAuth,
}

impl PolicyTemplateGrpc {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            auth: GrpcAuth::new(ctx.oauth_validator.clone()),
            ctx,
        }
    }
}

#[tonic::async_trait]
impl PolicyTemplateService for PolicyTemplateGrpc {
    async fn list(&self, request: Request<ListJsonRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let filter: PolicyTemplateFilter = match request.filter.as_deref().filter(|f| !f.is_empty()) {
            Some(json) => GrpcJson::parse(json, "filter")?,
            None => PolicyTemplateFilter::default(),
        };
        let (page, sort) = PageParams::from_proto(
            request.limit.unwrap_or(0),
            request.cursor.as_deref().unwrap_or_default(),
            request.sort.as_deref().unwrap_or_default(),
        )?;
        let found = self
            .ctx
            .templates()
            .get_all_policy_templates(&user, &filter, &page, &sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&found)?))
    }

    async fn versions(&self, request: Request<IdRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id;
        let found = self
            .ctx
            .templates()
            .get_policies_template_by_id(&user, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&found)?))
    }

    async fn get(&self, request: Request<TemplateVersionRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let found = self
            .ctx
            .templates()
            .get_policies_template_by_version_and_id(&user, &request.id, &request.version)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&found)?))
    }

    async fn create(&self, request: Request<JsonDocument>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let template: NewPolicyTemplateDto = GrpcJson::parse(&request.into_inner().json, "json")?;
        let created = self
            .ctx
            .templates()
            .create_policy_template(&user, &template)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&created)?))
    }

    async fn delete(&self, request: Request<TemplateVersionRequest>) -> Result<Response<()>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        self.ctx
            .templates()
            .delete_policy_template_by_version_and_id(&user, &request.id, &request.version)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }

    async fn instantiate(&self, request: Request<JsonDocument>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let instantiation: NewPolicyInstantiationDto = GrpcJson::parse(&request.into_inner().json, "json")?;
        let offer = self
            .ctx
            .instantiation()
            .instantiate_policy(&user, &instantiation)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&OdrlPolicyView::assemble(offer))?))
    }
}
