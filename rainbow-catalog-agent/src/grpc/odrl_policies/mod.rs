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

use crate::entities::odrl_policies::{NewOdrlPolicyDto, OdrlPolicyEntityTrait};
use crate::grpc::api::catalog_agent::odrl_policy_entity_service_server::OdrlPolicyEntityService;
use crate::grpc::api::catalog_agent::{
    CreateOdrlPolicyRequest, DeleteByEntityIdRequest, DeleteByIdRequest, GetAllRequest,
    GetBatchRequest, GetByEntityIdRequest, GetByIdRequest, OdrlPolicy, OdrlPolicyListResponse,
    OdrlPolicyResponse,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct OdrlPolicyEntityGrpc {
    service: Arc<dyn OdrlPolicyEntityTrait>,
}

impl OdrlPolicyEntityGrpc {
    pub fn new(service: Arc<dyn OdrlPolicyEntityTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl OdrlPolicyEntityService for OdrlPolicyEntityGrpc {
    async fn get_all_odrl_offers(
        &self,
        request: Request<GetAllRequest>,
    ) -> Result<Response<OdrlPolicyListResponse>, Status> {
        let req = request.into_inner();
        let policies = self
            .service
            .get_all_odrl_offers(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_policies: Vec<OdrlPolicy> = policies.into_iter().map(Into::into).collect();

        Ok(Response::new(OdrlPolicyListResponse { policies: proto_policies }))
    }

    async fn get_batch_odrl_offers(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<OdrlPolicyListResponse>, Status> {
        let req = request.into_inner();
        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("One or more IDs are invalid URNs"))?;

        let policies = self
            .service
            .get_batch_odrl_offers(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_policies: Vec<OdrlPolicy> = policies.into_iter().map(Into::into).collect();

        Ok(Response::new(OdrlPolicyListResponse { policies: proto_policies }))
    }

    async fn get_all_odrl_offers_by_entity(
        &self,
        request: Request<GetByEntityIdRequest>,
    ) -> Result<Response<OdrlPolicyListResponse>, Status> {
        let req = request.into_inner();
        let entity_urn = Urn::from_str(&req.entity_id)
            .map_err(|_| Status::invalid_argument("Invalid Entity URN"))?;

        let policies = self
            .service
            .get_all_odrl_offers_by_entity(&entity_urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_policies: Vec<OdrlPolicy> = policies.into_iter().map(Into::into).collect();

        Ok(Response::new(OdrlPolicyListResponse { policies: proto_policies }))
    }

    async fn get_odrl_offer_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<OdrlPolicyResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        let policy_opt = self
            .service
            .get_odrl_offer_by_id(&urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match policy_opt {
            Some(dto) => Ok(Response::new(OdrlPolicyResponse { policy: Some(dto.into()) })),
            None => Err(Status::not_found("ODRL Policy not found")),
        }
    }

    async fn create_odrl_offer(
        &self,
        request: Request<CreateOdrlPolicyRequest>,
    ) -> Result<Response<OdrlPolicyResponse>, Status> {
        let req = request.into_inner();
        let new_policy_dto: NewOdrlPolicyDto = req.try_into()?;

        let created_dto = self
            .service
            .create_odrl_offer(&new_policy_dto)
            .await
            .map_err(|e| Status::internal(format!("Failed to create ODRL Policy: {}", e)))?;

        Ok(Response::new(OdrlPolicyResponse { policy: Some(created_dto.into()) }))
    }

    async fn delete_odrl_offer_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id).map_err(|_| Status::invalid_argument("Invalid URN"))?;

        self.service
            .delete_odrl_offer_by_id(&urn)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete ODRL Policy: {}", e)))?;

        Ok(Response::new(()))
    }

    async fn delete_odrl_offers_by_entity(
        &self,
        request: Request<DeleteByEntityIdRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.entity_id)
            .map_err(|_| Status::invalid_argument("Invalid Entity URN"))?;

        self.service.delete_odrl_offers_by_entity(&urn).await.map_err(|e| {
            Status::internal(format!("Failed to delete ODRL Policies by Entity: {}", e))
        })?;

        Ok(Response::new(()))
    }
}
