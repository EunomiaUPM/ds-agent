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

//! gRPC adapter for agreement management service.

use crate::entities::agreement::{AgreementDto, EditAgreementDto, NewAgreementDto};
use crate::grpc::api::negotiation_agent::negotiation_agent_agreements_service_server::NegotiationAgentAgreementsService;
use crate::grpc::api::negotiation_agent::{
    AgreementListResponse, AgreementResponse, CreateAgreementRequest, DeleteAgreementRequest,
    GetAgreementByIdRequest, GetAgreementByNegotiationMessageRequest,
    GetAgreementByNegotiationProcessRequest, GetAllAgreementsRequest, GetBatchAgreementsRequest,
    PutAgreementRequest,
};
use crate::grpc::{GrpcAuthHelper, IntoGrpcStatus};
use crate::services::agreement::AgreementServiceTrait;
use common::auth::OauthTokenValidator;
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::paginated_spec::Page;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentAgreementGrpc {
    service: Arc<dyn AgreementServiceTrait>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl NegotiationAgentAgreementGrpc {
    pub fn new(
        service: Arc<dyn AgreementServiceTrait>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self { service, validator }
    }

    async fn scope(&self, meta: &tonic::metadata::MetadataMap) -> Result<AccessScope, Status> {
        GrpcAuthHelper::extract_scope(&self.validator, meta).await
    }
}

#[tonic::async_trait]
impl NegotiationAgentAgreementsService for NegotiationAgentAgreementGrpc {
    async fn get_all_agreements(
        &self,
        request: Request<GetAllAgreementsRequest>,
    ) -> Result<Response<AgreementListResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let page = Page::new(req.limit.unwrap_or(20) as u32, None);
        let paginated = self
            .service
            .get_all(&scope, &Default::default(), &page, &Default::default())
            .await
            .map_err(|e| e.into_status())?;

        let proto_agreements = paginated
            .items
            .into_iter()
            .map(|view| {
                let dto: AgreementDto = view.into();
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
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid URN in batch: {e}")))?;

        let batch_req = BatchRequests { ids: urns };
        let views = self
            .service
            .batch(&scope, &batch_req)
            .await
            .map_err(|e| e.into_status())?;

        let proto_agreements = views
            .into_iter()
            .map(|view| {
                let dto: AgreementDto = view.into();
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
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;

        let view = self
            .service
            .get_one(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: AgreementDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        request: Request<GetAgreementByNegotiationProcessRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {e}")))?;

        let view = self
            .service
            .get_by_process(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: AgreementDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        request: Request<GetAgreementByNegotiationMessageRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.message_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Message ID URN: {e}")))?;

        let view = self
            .service
            .get_by_message(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        let dto: AgreementDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn create_agreement(
        &self,
        request: Request<CreateAgreementRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let new_agreement_dto: NewAgreementDto = req.try_into()?;

        let view = self
            .service
            .create(&scope, &new_agreement_dto)
            .await
            .map_err(|e| e.into_status())?;
        let dto: AgreementDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn put_agreement(
        &self,
        request: Request<PutAgreementRequest>,
    ) -> Result<Response<AgreementResponse>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;
        let edit_dto: EditAgreementDto = req.try_into()?;

        let view = self
            .service
            .edit(&scope, &urn, &edit_dto)
            .await
            .map_err(|e| e.into_status())?;
        let dto: AgreementDto = view.into();
        Ok(Response::new(dto.into()))
    }

    async fn delete_agreement(
        &self,
        request: Request<DeleteAgreementRequest>,
    ) -> Result<Response<()>, Status> {
        let (meta, _, req) = request.into_parts();
        let scope = self.scope(&meta).await?;
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {e}")))?;

        self.service
            .delete(&scope, &urn)
            .await
            .map_err(|e| e.into_status())?;
        Ok(Response::new(()))
    }
}
