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

use crate::entities::distributions::{
    DistributionEntityTrait, EditDistributionDto, NewDistributionDto,
};
use crate::grpc::api::catalog_agent::distribution_entity_service_server::DistributionEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDistributionRequest, DeleteByIdRequest, Distribution, DistributionListResponse,
    DistributionResponse, GetAllRequest, GetBatchRequest, GetByIdRequest, GetByParentIdRequest,
    GetDistributionByFormatRequest, PutDistributionRequest,
};
use crate::grpc::auth::{GrpcAuth, StatusMapper};
use common::auth::OauthTokenValidator;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

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
        request: Request<GetAllRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let paginated = self
            .service
            .get_all_distributions(&scope, &Default::default(), &page, &Default::default())
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_distributions: Vec<Distribution> =
            paginated.items.into_iter().map(Into::into).collect();

        Ok(Response::new(DistributionListResponse {
            distributions: proto_distributions,
        }))
    }

    async fn get_batch_distributions(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more IDs are invalid URNs"))?;

        let distributions = self
            .service
            .get_batch_distributions(&scope, &urns)
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_distributions: Vec<Distribution> =
            distributions.into_iter().map(Into::into).collect();

        Ok(Response::new(DistributionListResponse {
            distributions: proto_distributions,
        }))
    }

    async fn get_distributions_by_dataset_id(
        &self,
        request: Request<GetByParentIdRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let dataset_urn = Urn::from_str(&req.parent_id)
            .map_err(|_| Status::invalid_argument("Invalid Dataset URN"))?;

        let distributions = self
            .service
            .get_distributions_by_dataset_id(&scope, &dataset_urn)
            .await
            .map_err(StatusMapper::to_status)?;

        let proto_distributions: Vec<Distribution> =
            distributions.into_iter().map(Into::into).collect();

        Ok(Response::new(DistributionListResponse {
            distributions: proto_distributions,
        }))
    }

    async fn get_distribution_by_dataset_and_format(
        &self,
        request: Request<GetDistributionByFormatRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let dataset_urn = Urn::from_str(&req.dataset_id)
            .map_err(|_| Status::invalid_argument("Invalid Dataset URN"))?;

        let distribution_dto = self
            .service
            .get_distribution_by_dataset_id_and_dct_format(&scope, &dataset_urn, &req.dct_formats)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(distribution_dto.into()),
        }))
    }

    async fn get_distribution_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        let dto = self
            .service
            .get_distribution_by_id(&scope, &urn)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(dto.into()),
        }))
    }

    async fn create_distribution(
        &self,
        request: Request<CreateDistributionRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let new_distribution_dto: NewDistributionDto = req.try_into()?;

        let created_dto = self
            .service
            .create_distribution(&scope, &new_distribution_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(created_dto.into()),
        }))
    }

    async fn put_distribution_by_id(
        &self,
        request: Request<PutDistributionRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;
        let edit_dto: EditDistributionDto = req.into();

        let updated_dto = self
            .service
            .put_distribution_by_id(&scope, &urn, &edit_dto)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(updated_dto.into()),
        }))
    }

    async fn delete_distribution_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        self.service
            .delete_distribution_by_id(&scope, &urn)
            .await
            .map_err(StatusMapper::to_status)?;

        Ok(Response::new(()))
    }
}
