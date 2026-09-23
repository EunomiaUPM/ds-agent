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

use crate::grpc::api::catalog_agent::catalog_entity_service_server::CatalogEntityService;
use crate::grpc::api::catalog_agent::{
    CatalogListResponse, CatalogResponse, CreateCatalogRequest, DeleteByIdRequest, GetBatchRequest,
    GetByIdRequest, ListCatalogsRequest, PutCatalogRequest,
};
use crate::services::catalogs::CatalogServiceTrait;
use common::auth::grpc::GrpcAuth;
use common::auth::OauthTokenValidator;
use common::grpc::{IntoStatus, ListParams, ProtoField, ProtoFieldList};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct CatalogEntityGrpc {
    service: Arc<dyn CatalogServiceTrait>,
    auth: GrpcAuth,
}

impl CatalogEntityGrpc {
    pub fn new(
        service: Arc<dyn CatalogServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl CatalogEntityService for CatalogEntityGrpc {
    async fn get_all_catalogs(
        &self,
        request: Request<ListCatalogsRequest>,
    ) -> Result<Response<CatalogListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all_catalogs(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_catalogs(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<CatalogListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ids = request.into_inner().ids.urns("ids")?;
        let dtos = self
            .service
            .get_batch_catalogs(&scope, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_catalog_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let dto = self
            .service
            .get_catalog_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.into()))
    }

    async fn get_main_catalog(
        &self,
        request: Request<()>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = self
            .service
            .get_main_catalog(&scope)
            .await
            .map_err(Errors::into_status)?
            .ok_or_else(|| Status::not_found("main catalog not configured"))?;
        Ok(Response::new(dto.into()))
    }

    async fn create_catalog(
        &self,
        request: Request<CreateCatalogRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_catalog(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn create_main_catalog(
        &self,
        request: Request<CreateCatalogRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_main_catalog(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn put_catalog_by_id(
        &self,
        request: Request<PutCatalogRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let updated = self
            .service
            .put_catalog_by_id(&scope, &id, &req.into())
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(updated.into()))
    }

    async fn delete_catalog_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete_catalog_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
