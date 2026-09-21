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

use crate::entities::catalogs::{CatalogEntityTrait, EditCatalogDto, NewCatalogDto};
use crate::entities::filters::CatalogFilter;
use crate::grpc::api::catalog_agent::catalog_entity_service_server::CatalogEntityService;
use crate::grpc::api::catalog_agent::{
    Catalog, CatalogListResponse, CatalogResponse, CreateCatalogRequest, DeleteByIdRequest,
    GetAllCatalogsRequest, GetBatchRequest, GetByIdRequest, PutCatalogRequest,
};
use crate::grpc::auth::{GrpcAuth, StatusMapper};
use common::auth::OauthTokenValidator;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct CatalogEntityGrpc {
    service: Arc<dyn CatalogEntityTrait>,
    auth: GrpcAuth,
}

impl CatalogEntityGrpc {
    pub fn new(
        service: Arc<dyn CatalogEntityTrait>,
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
        request: Request<GetAllCatalogsRequest>,
    ) -> Result<Response<CatalogListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let filter = CatalogFilter {
            with_main_catalog: Some(req.with_main_catalog),
            ..Default::default()
        };
        let paginated = self
            .service
            .get_all_catalogs(&scope, &filter, &page, &Default::default())
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_catalogs: Vec<Catalog> = paginated.items.into_iter().map(Into::into).collect();

        Ok(Response::new(CatalogListResponse {
            catalogs: proto_catalogs,
        }))
    }

    async fn get_batch_catalogs(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<CatalogListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more IDs are invalid URNs"))?;

        let catalogs = self
            .service
            .get_batch_catalogs(&scope, &urns)
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_catalogs = catalogs.into_iter().map(Into::into).collect();

        Ok(Response::new(CatalogListResponse {
            catalogs: proto_catalogs,
        }))
    }

    async fn get_catalog_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        let dto = self
            .service
            .get_catalog_by_id(&scope, &urn)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(CatalogResponse {
            catalog: Some(dto.into()),
        }))
    }

    async fn get_main_catalog(
        &self,
        request: Request<()>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let catalog_opt = self
            .service
            .get_main_catalog(&scope)
            .await
            .map_err(StatusMapper::to_status)?;

        match catalog_opt {
            Some(dto) => Ok(Response::new(CatalogResponse {
                catalog: Some(dto.into()),
            })),
            None => Err(Status::not_found("Main catalog not configured")),
        }
    }

    async fn create_catalog(
        &self,
        request: Request<CreateCatalogRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let new_catalog_dto: NewCatalogDto = req.try_into()?;

        let created_dto = self
            .service
            .create_catalog(&scope, &new_catalog_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(CatalogResponse {
            catalog: Some(created_dto.into()),
        }))
    }

    async fn create_main_catalog(
        &self,
        request: Request<CreateCatalogRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let new_catalog_dto: NewCatalogDto = req.try_into()?;

        let created_dto = self
            .service
            .create_main_catalog(&scope, &new_catalog_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(CatalogResponse {
            catalog: Some(created_dto.into()),
        }))
    }

    async fn put_catalog_by_id(
        &self,
        request: Request<PutCatalogRequest>,
    ) -> Result<Response<CatalogResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;
        let edit_dto: EditCatalogDto = req.into();

        let updated_dto = self
            .service
            .put_catalog_by_id(&scope, &urn, &edit_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(CatalogResponse {
            catalog: Some(updated_dto.into()),
        }))
    }

    async fn delete_catalog_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        self.service
            .delete_catalog_by_id(&scope, &urn)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(()))
    }
}
