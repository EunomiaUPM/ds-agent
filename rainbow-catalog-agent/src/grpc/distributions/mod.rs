use crate::entities::distributions::{
    DistributionEntityTrait, EditDistributionDto, NewDistributionDto,
};
use crate::grpc::api::catalog_agent::distribution_entity_service_server::DistributionEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDistributionRequest, DeleteByIdRequest, Distribution, DistributionListResponse,
    DistributionResponse, GetAllRequest, GetBatchRequest, GetByIdRequest, GetByParentIdRequest,
    GetDistributionByFormatRequest, PutDistributionRequest,
};
use rainbow_common::dcat_formats::DctFormats;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct DistributionEntityGrpc {
    service: Arc<dyn DistributionEntityTrait>,
}

impl DistributionEntityGrpc {
    pub fn new(service: Arc<dyn DistributionEntityTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl DistributionEntityService for DistributionEntityGrpc {
    async fn get_all_distributions(
        &self,
        request: Request<GetAllRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let req = request.into_inner();
        let distributions = self
            .service
            .get_all_distributions(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_distributions: Vec<Distribution> =
            distributions.into_iter().map(Into::into).collect();

        Ok(Response::new(DistributionListResponse {
            distributions: proto_distributions,
        }))
    }

    async fn get_batch_distributions(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DistributionListResponse>, Status> {
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more IDs are invalid URNs"))?;

        let distributions = self
            .service
            .get_batch_distributions(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

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
        let req = request.into_inner();
        let dataset_urn = Urn::from_str(&req.parent_id)
            .map_err(|_| Status::invalid_argument("Invalid Dataset URN"))?;

        let distributions = self
            .service
            .get_distributions_by_dataset_id(&dataset_urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

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
        let req = request.into_inner();
        let dataset_urn = Urn::from_str(&req.dataset_id)
            .map_err(|_| Status::invalid_argument("Invalid Dataset URN"))?;

        let distribution_dto = self
            .service
            .get_distribution_by_dataset_id_and_dct_format(&dataset_urn, &req.dct_formats)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(distribution_dto.into()),
        }))
    }

    async fn get_distribution_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        let distribution_opt = self
            .service
            .get_distribution_by_id(&urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match distribution_opt {
            Some(dto) => Ok(Response::new(DistributionResponse { distribution: Some(dto.into()) })),
            None => Err(Status::not_found("Distribution not found")),
        }
    }

    async fn create_distribution(
        &self,
        request: Request<CreateDistributionRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let req = request.into_inner();
        let new_distribution_dto: NewDistributionDto = req.try_into()?;

        let created_dto = self
            .service
            .create_distribution(&new_distribution_dto)
            .await
            .map_err(|e| Status::internal(format!("Failed to create distribution: {}", e)))?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(created_dto.into()),
        }))
    }

    async fn put_distribution_by_id(
        &self,
        request: Request<PutDistributionRequest>,
    ) -> Result<Response<DistributionResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;
        let edit_dto: EditDistributionDto = req.into();

        let updated_dto = self
            .service
            .put_distribution_by_id(&urn, &edit_dto)
            .await
            .map_err(|e| Status::internal(format!("Failed to update distribution: {}", e)))?;

        Ok(Response::new(DistributionResponse {
            distribution: Some(updated_dto.into()),
        }))
    }

    async fn delete_distribution_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        self.service
            .delete_distribution_by_id(&urn)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete distribution: {}", e)))?;

        Ok(Response::new(()))
    }
}
