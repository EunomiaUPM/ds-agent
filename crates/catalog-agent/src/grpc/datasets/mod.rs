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

use crate::entities::datasets::{DatasetEntityTrait, EditDatasetDto, NewDatasetDto};
use crate::grpc::api::catalog_agent::dataset_entity_service_server::DatasetEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDatasetRequest, Dataset, DatasetListResponse, DatasetResponse, DeleteByIdRequest,
    GetAllRequest, GetBatchRequest, GetByIdRequest, GetByParentIdRequest, PutDatasetRequest,
};
use crate::grpc::auth::{GrpcAuth, StatusMapper};
use common::auth::OauthTokenValidator;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

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
        request: Request<GetAllRequest>,
    ) -> Result<Response<DatasetListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let paginated = self
            .service
            .get_all_datasets(&scope, &Default::default(), &page, &Default::default())
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_datasets: Vec<Dataset> = paginated.items.into_iter().map(Into::into).collect();

        Ok(Response::new(DatasetListResponse {
            datasets: proto_datasets,
        }))
    }

    async fn get_batch_datasets(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DatasetListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more IDs are invalid URNs"))?;

        let datasets = self
            .service
            .get_batch_datasets(&scope, &urns)
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_datasets: Vec<Dataset> = datasets.into_iter().map(Into::into).collect();

        Ok(Response::new(DatasetListResponse {
            datasets: proto_datasets,
        }))
    }

    async fn get_datasets_by_catalog_id(
        &self,
        request: Request<GetByParentIdRequest>,
    ) -> Result<Response<DatasetListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let catalog_urn = Urn::from_str(&req.parent_id)
            .map_err(|_| Status::invalid_argument("Invalid Catalog URN"))?;

        let datasets = self
            .service
            .get_datasets_by_catalog_id(&scope, &catalog_urn)
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_datasets: Vec<Dataset> = datasets.into_iter().map(Into::into).collect();

        Ok(Response::new(DatasetListResponse {
            datasets: proto_datasets,
        }))
    }

    async fn get_dataset_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DatasetResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        let dto = self
            .service
            .get_dataset_by_id(&scope, &urn)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DatasetResponse {
            dataset: Some(dto.into()),
        }))
    }

    async fn create_dataset(
        &self,
        request: Request<CreateDatasetRequest>,
    ) -> Result<Response<DatasetResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let new_dataset_dto: NewDatasetDto = req.try_into()?;

        let created_dto = self
            .service
            .create_dataset(&scope, &new_dataset_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DatasetResponse {
            dataset: Some(created_dto.into()),
        }))
    }

    async fn put_dataset_by_id(
        &self,
        request: Request<PutDatasetRequest>,
    ) -> Result<Response<DatasetResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;
        let edit_dto: EditDatasetDto = req.into();

        let updated_dto = self
            .service
            .put_dataset_by_id(&scope, &urn, &edit_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DatasetResponse {
            dataset: Some(updated_dto.into()),
        }))
    }

    async fn delete_dataset_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        self.service
            .delete_dataset_by_id(&scope, &urn)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(()))
    }
}
