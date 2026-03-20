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

use crate::entities::data_services::{
    DataServiceEntityTrait, EditDataServiceDto, NewDataServiceDto,
};
use crate::grpc::api::catalog_agent::data_service_entity_service_server::DataServiceEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDataServiceRequest, DataService, DataServiceListResponse, DataServiceResponse,
    DeleteByIdRequest, GetAllRequest, GetBatchRequest, GetByIdRequest, GetByParentIdRequest,
    PutDataServiceRequest,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct DataServiceEntityGrpc {
    service: Arc<dyn DataServiceEntityTrait>,
}

impl DataServiceEntityGrpc {
    pub fn new(service: Arc<dyn DataServiceEntityTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl DataServiceEntityService for DataServiceEntityGrpc {
    async fn get_all_data_services(
        &self,
        request: Request<GetAllRequest>,
    ) -> Result<Response<DataServiceListResponse>, Status> {
        let req = request.into_inner();
        let data_services = self
            .service
            .get_all_data_services(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_services: Vec<DataService> = data_services.into_iter().map(Into::into).collect();

        Ok(Response::new(DataServiceListResponse {
            data_services: proto_services,
        }))
    }

    async fn get_batch_data_services(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DataServiceListResponse>, Status> {
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more IDs are invalid URNs"))?;

        let data_services = self
            .service
            .get_batch_data_services(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_services: Vec<DataService> = data_services.into_iter().map(Into::into).collect();

        Ok(Response::new(DataServiceListResponse {
            data_services: proto_services,
        }))
    }

    async fn get_data_services_by_catalog_id(
        &self,
        request: Request<GetByParentIdRequest>,
    ) -> Result<Response<DataServiceListResponse>, Status> {
        let req = request.into_inner();
        let catalog_urn = Urn::from_str(&req.parent_id)
            .map_err(|_| Status::invalid_argument("Invalid Catalog URN"))?;

        let data_services = self
            .service
            .get_data_services_by_catalog_id(&catalog_urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_services: Vec<DataService> = data_services.into_iter().map(Into::into).collect();

        Ok(Response::new(DataServiceListResponse {
            data_services: proto_services,
        }))
    }

    async fn get_data_service_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        let data_service_opt = self
            .service
            .get_data_service_by_id(&urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match data_service_opt {
            Some(dto) => Ok(Response::new(DataServiceResponse {
                data_service: Some(dto.into()),
            })),
            None => Err(Status::not_found("DataService not found")),
        }
    }

    async fn get_main_data_service(
        &self,
        request: Request<()>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let _req = request.into_inner();

        let data_service_opt = self
            .service
            .get_main_data_service()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match data_service_opt {
            Some(dto) => Ok(Response::new(DataServiceResponse {
                data_service: Some(dto.into()),
            })),
            None => Err(Status::not_found("DataService not found")),
        }
    }

    async fn create_data_service(
        &self,
        request: Request<CreateDataServiceRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let req = request.into_inner();
        let new_data_service_dto: NewDataServiceDto = req.try_into()?;

        let created_dto = self
            .service
            .create_data_service(&new_data_service_dto)
            .await
            .map_err(|e| Status::internal(format!("Failed to create data service: {}", e)))?;

        Ok(Response::new(DataServiceResponse {
            data_service: Some(created_dto.into()),
        }))
    }

    async fn create_main_main_catalog(
        &self,
        request: Request<CreateDataServiceRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let req = request.into_inner();
        let new_data_service_dto: NewDataServiceDto = req.try_into()?;

        let created_dto = self
            .service
            .create_main_data_service(&new_data_service_dto)
            .await
            .map_err(|e| Status::internal(format!("Failed to create data service: {}", e)))?;

        Ok(Response::new(DataServiceResponse {
            data_service: Some(created_dto.into()),
        }))
    }

    async fn put_data_service_by_id(
        &self,
        request: Request<PutDataServiceRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;
        let edit_dto: EditDataServiceDto = req.into();

        let updated_dto = self
            .service
            .put_data_service_by_id(&urn, &edit_dto)
            .await
            .map_err(|e| Status::internal(format!("Failed to update data service: {}", e)))?;

        Ok(Response::new(DataServiceResponse {
            data_service: Some(updated_dto.into()),
        }))
    }

    async fn delete_data_service_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        self.service
            .delete_data_service_by_id(&urn)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete data service: {}", e)))?;

        Ok(Response::new(()))
    }
}
