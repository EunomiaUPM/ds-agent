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

use crate::entities::odrl_policies::OdrlPolicyEntityTrait;
use crate::grpc::api::catalog_agent::odrl_policy_entity_service_server::OdrlPolicyEntityService;
use crate::grpc::api::catalog_agent::{
    CreateOdrlPolicyRequest, DeleteByEntityIdRequest, DeleteByIdRequest, GetBatchRequest,
    GetByEntityIdRequest, GetByIdRequest, ListOdrlPoliciesRequest, OdrlPolicyListResponse,
    OdrlPolicyResponse,
};
use common::auth::grpc::GrpcAuth;
use common::auth::OauthTokenValidator;
use common::grpc::{IntoStatus, ListParams, ProtoField, ProtoFieldList};
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

pub struct OdrlPolicyEntityGrpc {
    service: Arc<dyn OdrlPolicyEntityTrait>,
    auth: GrpcAuth,
}

impl OdrlPolicyEntityGrpc {
    pub fn new(
        service: Arc<dyn OdrlPolicyEntityTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            service,
            auth: GrpcAuth::new(validator),
        }
    }
}

#[tonic::async_trait]
impl OdrlPolicyEntityService for OdrlPolicyEntityGrpc {
    async fn get_all_odrl_offers(
        &self,
        request: Request<ListOdrlPoliciesRequest>,
    ) -> Result<Response<OdrlPolicyListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let params = ListParams::try_from(request.into_inner())?;
        let result = self
            .service
            .get_all_odrl_offers(&scope, &params.filter, &params.page, &params.sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(result.into()))
    }

    async fn get_batch_odrl_offers(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<OdrlPolicyListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let ids = request.into_inner().ids.urns("ids")?;
        let dtos = self
            .service
            .get_batch_odrl_offers(&scope, &ids)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_all_odrl_offers_by_entity(
        &self,
        request: Request<GetByEntityIdRequest>,
    ) -> Result<Response<OdrlPolicyListResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let entity_id = request.into_inner().entity_id.urn("entity_id")?;
        let dtos = self
            .service
            .get_all_odrl_offers_by_entity(&scope, &entity_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dtos.into()))
    }

    async fn get_odrl_offer_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<OdrlPolicyResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let dto = self
            .service
            .get_odrl_offer_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(dto.into()))
    }

    async fn create_odrl_offer(
        &self,
        request: Request<CreateOdrlPolicyRequest>,
    ) -> Result<Response<OdrlPolicyResponse>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let dto = request.into_inner().try_into()?;
        let created = self
            .service
            .create_odrl_offer(&scope, &dto)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(created.into()))
    }

    async fn delete_odrl_offer_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.service
            .delete_odrl_offer_by_id(&scope, &id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }

    async fn delete_odrl_offers_by_entity(
        &self,
        request: Request<DeleteByEntityIdRequest>,
    ) -> Result<Response<()>, Status> {
        let scope = self.auth.scope(request.metadata()).await?;
        let entity_id = request.into_inner().entity_id.urn("entity_id")?;
        self.service
            .delete_odrl_offers_by_entity(&scope, &entity_id)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
