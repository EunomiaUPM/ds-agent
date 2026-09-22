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

use crate::entities::distributions::DistributionEntityTrait;
use crate::grpc::api::catalog_agent::distribution_entity_service_server::DistributionEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDistributionRequest, DeleteByIdRequest, DistributionListResponse, DistributionResponse,
    GetBatchRequest, GetByIdRequest, GetByParentIdRequest, GetDistributionByFormatRequest,
    ListDistributionsRequest, PutDistributionRequest,
};
use common::auth::grpc::GrpcAuth;
use common::auth::OauthTokenValidator;
use common::grpc::{IntoStatus, ListParams, ProtoField, ProtoFieldList};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct DistributionEntityGrpc {
    service: Arc<dyn DistributionEntityTrait>,
    auth: GrpcAuth,
}

impl DistributionEntityGrpc {
    pub fn new(
        service: Arc<dyn DistributionEntityTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl DistributionEntityService for DistributionEntityGrpc {
    async fn get_all_distributions(
        &self,
        request: Request<ListDistributionsRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all_distributions(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_distributions(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ids = request.into_inner().ids.urns("ids")?;
        let dtos = self
            .service
            .get_batch_distributions(&scope, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_distributions_by_dataset_id(
        &self,
        request: Request<GetByParentIdRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dataset_id = request.into_inner().parent_id.urn("parent_id")?;
        let dtos = self
            .service
            .get_distributions_by_dataset_id(&scope, &dataset_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_distribution_by_dataset_and_format(
        &self,
        request: Request<GetDistributionByFormatRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let dataset_id = req.dataset_id.urn("dataset_id")?;
        let dto = self
            .service
            .get_distribution_by_dataset_id_and_dct_format(&scope, &dataset_id, &req.dct_formats)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.into()))
    }

    async fn get_distribution_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let dto = self
            .service
            .get_distribution_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.into()))
    }

    async fn create_distribution(
        &self,
        request: Request<CreateDistributionRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_distribution(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn put_distribution_by_id(
        &self,
        request: Request<PutDistributionRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let updated = self
            .service
            .put_distribution_by_id(&scope, &id, &req.into())
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(updated.into()))
    }

    async fn delete_distribution_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete_distribution_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
