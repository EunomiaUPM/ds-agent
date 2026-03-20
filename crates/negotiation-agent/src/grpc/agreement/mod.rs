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

use crate::entities::agreement::{
    EditAgreementDto, NegotiationAgentAgreementsTrait, NewAgreementDto,
};
use crate::grpc::api::negotiation_agent::negotiation_agent_agreements_service_server::NegotiationAgentAgreementsService;
use crate::grpc::api::negotiation_agent::{
    AgreementListResponse, AgreementResponse, CreateAgreementRequest, DeleteAgreementRequest,
    GetAgreementByIdRequest, GetAgreementByNegotiationMessageRequest,
    GetAgreementByNegotiationProcessRequest, GetAllAgreementsRequest, GetBatchAgreementsRequest,
    PutAgreementRequest,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentAgreementGrpc {
    service: Arc<dyn NegotiationAgentAgreementsTrait>,
}

impl NegotiationAgentAgreementGrpc {
    pub fn new(service: Arc<dyn NegotiationAgentAgreementsTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl NegotiationAgentAgreementsService for NegotiationAgentAgreementGrpc {
    async fn get_all_agreements(
        &self,
        request: Request<GetAllAgreementsRequest>,
    ) -> Result<Response<AgreementListResponse>, Status> {
        let req = request.into_inner();

        let agreements = self
            .service
            .get_all_agreements(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_agreements = agreements
            .into_iter()
            .map(|dto| {
                let response: AgreementResponse = dto.into();
                response.agreement.unwrap()
            })
            .collect();

        Ok(Response::new(AgreementListResponse {
            agreements: proto_agreements,
        }))
    }

    async fn get_batch_agreements(
        &self,
        request: Request<GetBatchAgreementsRequest>,
    ) -> Result<Response<AgreementListResponse>, Status> {
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid URN in batch: {}", e)))?;

        let agreements = self
            .service
            .get_batch_agreements(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_agreements = agreements
            .into_iter()
            .map(|dto| {
                let response: AgreementResponse = dto.into();
                response.agreement.unwrap()
            })
            .collect();

        Ok(Response::new(AgreementListResponse {
            agreements: proto_agreements,
        }))
    }

    async fn get_agreement_by_id(
        &self,
        request: Request<GetAgreementByIdRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.get_agreement_by_id(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Agreement not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        request: Request<GetAgreementByNegotiationProcessRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {}", e)))?;

        match self
            .service
            .get_agreement_by_negotiation_process(&urn)
            .await
        {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found(
                "Agreement not found for this negotiation process",
            )),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        request: Request<GetAgreementByNegotiationMessageRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.message_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Message ID URN: {}", e)))?;

        match self
            .service
            .get_agreement_by_negotiation_message(&urn)
            .await
        {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found(
                "Agreement not found for this negotiation message",
            )),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_agreement(
        &self,
        request: Request<CreateAgreementRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let req = request.into_inner();

        let new_agreement_dto: NewAgreementDto = req.try_into()?;

        match self.service.create_agreement(&new_agreement_dto).await {
            Ok(dto) => Ok(Response::new(dto.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn put_agreement(
        &self,
        request: Request<PutAgreementRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        let edit_dto: EditAgreementDto = req.try_into()?;

        match self.service.put_agreement(&urn, &edit_dto).await {
            Ok(dto) => Ok(Response::new(dto.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_agreement(
        &self,
        request: Request<DeleteAgreementRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.delete_agreement(&urn).await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
