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

use crate::entities::datasets::DatasetEntityTrait;
use crate::grpc::api::catalog_agent::dataset_entity_service_server::DatasetEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDatasetRequest, DatasetListResponse, DatasetResponse, DeleteByIdRequest, GetBatchRequest,
    GetByIdRequest, GetByParentIdRequest, ListDatasetsRequest, PutDatasetRequest,
};
use common::auth::grpc::GrpcAuth;
use common::auth::OauthTokenValidator;
use common::grpc::{IntoStatus, ListParams, ProtoField, ProtoFieldList};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct DatasetEntityGrpc {
    service: Arc<dyn DatasetEntityTrait>,
    auth: GrpcAuth,
}

impl DatasetEntityGrpc {
    pub fn new(
        service: Arc<dyn DatasetEntityTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl DatasetEntityService for DatasetEntityGrpc {
    async fn get_all_datasets(
        &self,
        request: Request<ListDatasetsRequest>,
    ) -> Result<Response<DatasetListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all_datasets(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_datasets(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DatasetListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ids = request.into_inner().ids.urns("ids")?;
        let dtos = self
            .service
            .get_batch_datasets(&scope, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_datasets_by_catalog_id(
        &self,
        request: Request<GetByParentIdRequest>,
    ) -> Result<Response<DatasetListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let catalog_id = request.into_inner().parent_id.urn("parent_id")?;
        let dtos = self
            .service
            .get_datasets_by_catalog_id(&scope, &catalog_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_dataset_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DatasetResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let dto = self
            .service
            .get_dataset_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.into()))
    }

    async fn create_dataset(
        &self,
        request: Request<CreateDatasetRequest>,
    ) -> Result<Response<DatasetResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_dataset(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn put_dataset_by_id(
        &self,
        request: Request<PutDatasetRequest>,
    ) -> Result<Response<DatasetResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let updated = self
            .service
            .put_dataset_by_id(&scope, &id, &req.into())
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(updated.into()))
    }

    async fn delete_dataset_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete_dataset_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
