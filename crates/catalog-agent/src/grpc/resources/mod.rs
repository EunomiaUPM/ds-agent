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

//! The resource RPCs over the resource and doctor services.

use std::str::FromStr;
use std::sync::Arc;

use common::grpc::{IntoStatus, PageParams, ProtoField, ProtoFieldList};
use common::oauth::grpc::GrpcAuth;
use common::paginated_spec::Paginated;
use serde_json::json;
use tonic::{Request, Response, Status};
use ymir::errors::Errors;
use ymir::types::participants::Visibility;

use crate::entities::admin_document::AdminDocument;
use crate::entities::filters::ResourceFilter;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::grpc::api::catalog_agent::catalog_resource_service_server::CatalogResourceService;
use crate::grpc::api::catalog_agent::{
    BatchResourcesRequest, ChildrenRequest, CreateResourceRequest, DeleteResourceRequest,
    DoctorRequest, JsonDocument, ListResourcesRequest, MainRequest, ReplaceResourceRequest,
    Resource, ResourceList, ResourceRef,
};
use crate::grpc::GrpcJson;
use crate::setup::context::AppContext;

pub struct CatalogResourceGrpc {
    ctx: Arc<AppContext>,
    auth: GrpcAuth,
}

impl CatalogResourceGrpc {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            auth: GrpcAuth::new(ctx.oauth_validator.clone()),
            ctx,
        }
    }

    fn kind(kind: &str) -> Result<ResourceKind, Status> {
        ResourceKind::from_str(kind).map_err(|e| Status::invalid_argument(format!("kind: {e}")))
    }

    fn visibility(visibility: Option<&str>) -> Result<Option<Visibility>, Status> {
        visibility
            .filter(|v| !v.is_empty())
            .map(|v| serde_json::from_value(json!(v)).map_err(|_| Status::invalid_argument("visibility: Private, Anonymous or Public")))
            .transpose()
    }

    fn resource(resource: &CatalogResource) -> Resource {
        let owner = resource.owner();
        Resource {
            id: resource.id().to_string(),
            kind: resource.kind().as_str().to_string(),
            document: AdminDocument::render(resource).to_string(),
            version: resource.version(),
            visibility: serde_json::to_value(&owner.visibility)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default(),
            owner: owner.user_id.clone(),
        }
    }

    fn list(resources: &[CatalogResource]) -> ResourceList {
        ResourceList {
            items: resources.iter().map(Self::resource).collect(),
            next_cursor: None,
            total: None,
        }
    }

    fn page(page: Paginated<CatalogResource>) -> ResourceList {
        ResourceList {
            items: page.items.iter().map(Self::resource).collect(),
            next_cursor: page.next_cursor,
            total: page.total,
        }
    }
}

#[tonic::async_trait]
impl CatalogResourceService for CatalogResourceGrpc {
    async fn list(&self, request: Request<ListResourcesRequest>) -> Result<Response<ResourceList>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let mut filter: ResourceFilter = match request.filter.as_deref().filter(|f| !f.is_empty()) {
            Some(json) => GrpcJson::parse(json, "filter")?,
            None => ResourceFilter::default(),
        };
        filter.kind = Some(Self::kind(&request.kind)?);
        let (page, sort) = PageParams::from_proto(
            request.limit.unwrap_or(0),
            request.cursor.as_deref().unwrap_or_default(),
            request.sort.as_deref().unwrap_or_default(),
        )?;
        let found = self
            .ctx
            .resources()
            .list(&user, &filter, &page, &sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::page(found)))
    }

    async fn batch(&self, request: Request<BatchResourcesRequest>) -> Result<Response<ResourceList>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let ids = request.ids.urns("ids")?;
        let found = self
            .ctx
            .resources()
            .batch(&user, Self::kind(&request.kind)?, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::list(&found)))
    }

    async fn get(&self, request: Request<ResourceRef>) -> Result<Response<Resource>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let found = self
            .ctx
            .resources()
            .get(&user, Self::kind(&request.kind)?, &request.id.urn("id")?)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::resource(&found)))
    }

    async fn main(&self, request: Request<MainRequest>) -> Result<Response<Resource>, Status> {
        self.auth.user(request.metadata()).await?;
        let kind = Self::kind(&request.into_inner().kind)?;
        match self.ctx.resources().main(kind).await.map_err(Errors::into_status)? {
            Some(found) => Ok(Response::new(Self::resource(&found))),
            None => Err(Status::not_found(format!("the connector has no main {}", kind.term()))),
        }
    }

    async fn children(&self, request: Request<ChildrenRequest>) -> Result<Response<ResourceList>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let found = self
            .ctx
            .resources()
            .children(&user, Self::kind(&request.kind)?, &request.parent_id.urn("parent_id")?)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::list(&found)))
    }

    async fn create(&self, request: Request<CreateResourceRequest>) -> Result<Response<Resource>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let created = self
            .ctx
            .resources()
            .create_document(
                &user,
                Self::kind(&request.kind)?,
                request.document.json("document")?,
                Self::visibility(request.visibility.as_deref())?,
                false,
            )
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::resource(&created)))
    }

    async fn create_main(&self, request: Request<CreateResourceRequest>) -> Result<Response<Resource>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let created = self
            .ctx
            .resources()
            .create_document(
                &user,
                Self::kind(&request.kind)?,
                request.document.opt_json("document")?.unwrap_or_else(|| json!({})),
                None,
                true,
            )
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::resource(&created)))
    }

    async fn replace(&self, request: Request<ReplaceResourceRequest>) -> Result<Response<Resource>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let replaced = self
            .ctx
            .resources()
            .replace_document(
                &user,
                Self::kind(&request.kind)?,
                &request.id.urn("id")?,
                request.document.json("document")?,
                Self::visibility(request.visibility.as_deref())?,
                request.if_match,
            )
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(Self::resource(&replaced)))
    }

    async fn delete(&self, request: Request<DeleteResourceRequest>) -> Result<Response<()>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        self.ctx
            .resources()
            .delete(&user, Self::kind(&request.kind)?, &request.id.urn("id")?, request.if_match)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }

    async fn doctor(&self, request: Request<DoctorRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let report = self
            .ctx
            .doctor()
            .diagnose(
                &user,
                Self::kind(&request.kind)?,
                &request.id.urn("id")?,
                request.profile.filter(|p| !p.is_empty()),
                request.deep,
            )
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&report)?))
    }
}
