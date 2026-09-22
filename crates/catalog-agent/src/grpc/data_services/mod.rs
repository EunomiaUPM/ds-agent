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

use crate::entities::data_services::DataServiceEntityTrait;
use crate::grpc::api::catalog_agent::data_service_entity_service_server::DataServiceEntityService;
use crate::grpc::api::catalog_agent::{
    CreateDataServiceRequest, DataServiceListResponse, DataServiceResponse, DeleteByIdRequest,
    GetBatchRequest, GetByIdRequest, GetByParentIdRequest, ListDataServicesRequest,
    PutDataServiceRequest,
};
use common::auth::grpc::GrpcAuth;
use common::auth::OauthTokenValidator;
use common::grpc::{IntoStatus, ListParams, ProtoField, ProtoFieldList};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct DataServiceEntityGrpc {
    service: Arc<dyn DataServiceEntityTrait>,
    auth: GrpcAuth,
}

impl DataServiceEntityGrpc {
    pub fn new(
        service: Arc<dyn DataServiceEntityTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl DataServiceEntityService for DataServiceEntityGrpc {
    async fn get_all_data_services(
        &self,
        request: Request<ListDataServicesRequest>,
    ) -> Result<Response<DataServiceListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all_data_services(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_data_services(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<DataServiceListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ids = request.into_inner().ids.urns("ids")?;
        let dtos = self
            .service
            .get_batch_data_services(&scope, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_data_services_by_catalog_id(
        &self,
        request: Request<GetByParentIdRequest>,
    ) -> Result<Response<DataServiceListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let catalog_id = request.into_inner().parent_id.urn("parent_id")?;
        let dtos = self
            .service
            .get_data_services_by_catalog_id(&scope, &catalog_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_data_service_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let dto = self
            .service
            .get_data_service_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.into()))
    }

    async fn get_main_data_service(
        &self,
        request: Request<()>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = self
            .service
            .get_main_data_service(&scope)
            .await
            .map_err(Errors::into_status)?
            .ok_or_else(|| Status::not_found("main data service not configured"))?;
        Ok(Response::new(dto.into()))
    }

    async fn create_data_service(
        &self,
        request: Request<CreateDataServiceRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_data_service(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn create_main_main_catalog(
        &self,
        request: Request<CreateDataServiceRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_main_data_service(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn put_data_service_by_id(
        &self,
        request: Request<PutDataServiceRequest>,
    ) -> Result<Response<DataServiceResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let req = request.into_inner();
        let id = req.id.urn("id")?;
        let updated = self
            .service
            .put_data_service_by_id(&scope, &id, &req.into())
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(updated.into()))
    }

    async fn delete_data_service_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete_data_service_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
