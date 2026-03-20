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

use crate::entities::offer::{NegotiationAgentOffersTrait, NewOfferDto};
use crate::grpc::api::negotiation_agent::negotiation_agent_offers_service_server::NegotiationAgentOffersService;
use crate::grpc::api::negotiation_agent::{
    CreateOfferRequest, DeleteOfferRequest, GetAllOffersRequest, GetBatchOffersRequest,
    GetOfferByIdRequest, GetOfferByNegotiationMessageRequest, GetOfferByOfferIdRequest,
    GetOffersByNegotiationProcessRequest, OfferListResponse, OfferResponse,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use urn::Urn;

pub struct NegotiationAgentOfferGrpc {
    service: Arc<dyn NegotiationAgentOffersTrait>,
}

impl NegotiationAgentOfferGrpc {
    pub fn new(service: Arc<dyn NegotiationAgentOffersTrait>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl NegotiationAgentOffersService for NegotiationAgentOfferGrpc {
    async fn get_all_offers(
        &self,
        request: Request<GetAllOffersRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let req = request.into_inner();

        let offers = self
            .service
            .get_all_offers(req.limit, req.page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_offers = offers
            .into_iter()
            .map(|dto| {
                let response: OfferResponse = dto.into();
                response.offer.unwrap()
            })
            .collect();

        Ok(Response::new(OfferListResponse {
            offers: proto_offers,
        }))
    }

    async fn get_batch_offers(
        &self,
        request: Request<GetBatchOffersRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let req = request.into_inner();

        let urns: Vec<Urn> = req
            .ids
            .iter()
            .map(|id| Urn::from_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid URN in batch: {}", e)))?;

        let offers = self
            .service
            .get_batch_offers(&urns)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_offers = offers
            .into_iter()
            .map(|dto| {
                let response: OfferResponse = dto.into();
                response.offer.unwrap()
            })
            .collect();

        Ok(Response::new(OfferListResponse {
            offers: proto_offers,
        }))
    }

    async fn get_offers_by_negotiation_process(
        &self,
        request: Request<GetOffersByNegotiationProcessRequest>,
    ) -> Result<Response<OfferListResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process ID URN: {}", e)))?;

        let offers = self
            .service
            .get_offers_by_negotiation_process(&urn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_offers = offers
            .into_iter()
            .map(|dto| {
                let response: OfferResponse = dto.into();
                response.offer.unwrap()
            })
            .collect();

        Ok(Response::new(OfferListResponse {
            offers: proto_offers,
        }))
    }

    async fn get_offer_by_id(
        &self,
        request: Request<GetOfferByIdRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.get_offer_by_id(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Offer not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_offer_by_negotiation_message(
        &self,
        request: Request<GetOfferByNegotiationMessageRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.message_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Message ID URN: {}", e)))?;

        match self.service.get_offer_by_negotiation_message(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Offer not found for this message")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_offer_by_offer_id(
        &self,
        request: Request<GetOfferByOfferIdRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let req = request.into_inner();
        // Asumimos que offer_id (externo) también se maneja como URN en la capa de servicio
        // según la firma del trait: get_offer_by_offer_id(&self, id: &Urn)
        let urn = Urn::from_str(&req.offer_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Offer ID URN: {}", e)))?;

        match self.service.get_offer_by_offer_id(&urn).await {
            Ok(Some(dto)) => Ok(Response::new(dto.into())),
            Ok(None) => Err(Status::not_found("Offer not found by external ID")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_offer(
        &self,
        request: Request<CreateOfferRequest>,
    ) -> Result<Response<OfferResponse>, Status> {
        let req = request.into_inner();

        // Usamos el TryFrom definido en los mappers para convertir Request -> DTO
        let new_offer_dto: NewOfferDto = req.try_into()?;

        match self.service.create_offer(&new_offer_dto).await {
            Ok(dto) => Ok(Response::new(dto.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_offer(
        &self,
        request: Request<DeleteOfferRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let urn = Urn::from_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid ID URN: {}", e)))?;

        match self.service.delete_offer(&urn).await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => {
                // Si el error es "NotFound" (gestionado por CommonErrors usualmente),
                // podríamos querer devolver Status::not_found, pero aquí genérico:
                Err(Status::internal(e.to_string()))
            }
        }
    }
}
