/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::entities::agreement::{AgreementDto, EditAgreementDto, NewAgreementDto};
use crate::entities::negotiation_message::{NegotiationMessageDto, NewNegotiationMessageDto};
use crate::entities::negotiation_process::{
    EditNegotiationProcessDto, NegotiationProcessDto, NewNegotiationProcessDto,
};
use crate::entities::offer::{NewOfferDto, OfferDto};

use crate::grpc::api::negotiation_agent::{
    Agreement as ProtoAgreement, NegotiationMessage as ProtoMessage,
    NegotiationProcess as ProtoProcess, Offer as ProtoOffer,
};
use crate::grpc::api::negotiation_agent::{
    AgreementResponse, CreateAgreementRequest, PutAgreementRequest,
};
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationMessageRequest, NegotiationMessageResponse,
};
use crate::grpc::api::negotiation_agent::{
    CreateNegotiationProcessRequest, GetBatchNegotiationProcessesRequest,
    NegotiationProcessResponse, PutNegotiationProcessRequest,
};
use crate::grpc::api::negotiation_agent::{CreateOfferRequest, OfferResponse};

use common::batch_requests::BatchRequests;
use prost_types::value::Kind;
use prost_types::{ListValue, Struct, Value as ProstValue};
use serde_json::{Map, Value as JsonValue};
use std::str::FromStr;
use tonic::Status;
use urn::Urn;

// =============================================================================
// PROCESSES MAPPERS
// =============================================================================

// PROTO -> DOMAIN (Request -> DTO)
impl TryFrom<CreateNegotiationProcessRequest> for NewNegotiationProcessDto {
    type Error = Status;

    fn try_from(proto: CreateNegotiationProcessRequest) -> Result<Self, Self::Error> {
        let id_urn = if let Some(id) = proto.id {
            Some(
                Urn::from_str(&id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid Process URN: {}", e)))?,
            )
        } else {
            None
        };

        let properties = if let Some(props) = proto.properties {
            Some(prost_struct_to_serde(props))
        } else {
            None
        };

        let identifiers = if proto.identifiers.is_empty() {
            None
        } else {
            Some(proto.identifiers)
        };

        Ok(NewNegotiationProcessDto {
            id: id_urn,
            state: proto.state,
            state_attribute: proto.state_attribute,
            associated_agent_peer: proto.associated_agent_peer,
            protocol: proto.protocol,
            callback_address: proto.callback_address,
            role: proto.role,
            properties,
            identifiers,
        })
    }
}

impl TryFrom<PutNegotiationProcessRequest> for EditNegotiationProcessDto {
    type Error = Status;

    fn try_from(proto: PutNegotiationProcessRequest) -> Result<Self, Self::Error> {
        let properties = proto.properties.map(prost_struct_to_serde);
        let error_details = proto.error_details.map(prost_struct_to_serde);
        let identifiers = if proto.identifiers.is_empty() {
            None
        } else {
            Some(proto.identifiers)
        };

        Ok(EditNegotiationProcessDto {
            state: proto.state,
            state_attribute: proto.state_attribute,
            properties,
            error_details,
            identifiers,
        })
    }
}

impl From<GetBatchNegotiationProcessesRequest> for BatchRequests {
    fn from(proto: GetBatchNegotiationProcessesRequest) -> Self {
        BatchRequests {
            ids: proto
                .ids
                .iter()
                .map(|i| Urn::from_str(i).unwrap_or_else(|_| Urn::from_str("urn:error").unwrap()))
                .collect(),
        }
    }
}

// DOMAIN -> PROTO (DTO -> Response)
impl From<NegotiationProcessDto> for NegotiationProcessResponse {
    fn from(dto: NegotiationProcessDto) -> Self {
        let inner = dto.inner;

        // Convertir Sub-entidades a Proto
        let messages: Vec<ProtoMessage> = dto
            .messages
            .into_iter()
            .map(model_message_to_proto)
            .collect();
        let offers: Vec<ProtoOffer> = dto.offers.into_iter().map(model_offer_to_proto).collect();
        let agreements: Vec<ProtoAgreement> = dto
            .agreement
            .into_iter()
            .map(model_agreement_to_proto)
            .collect();

        let properties = serde_to_prost_struct(inner.properties);
        let error_details = inner.error_details.map(serde_to_prost_struct);

        let process = ProtoProcess {
            id: inner.id,
            state: inner.state,
            state_attribute: inner.state_attribute,
            associated_agent_peer: inner.associated_agent_peer,
            protocol: inner.protocol,
            callback_address: inner.callback_address,
            role: inner.role,
            properties: Some(properties),
            error_details,
            created_at: inner.created_at.to_rfc3339(),
            updated_at: inner.updated_at.map(|d| d.to_rfc3339()),
            identifiers: dto.identifiers,
            messages,
            offers,
            agreements,
        };

        NegotiationProcessResponse {
            process: Some(process),
        }
    }
}

// =============================================================================
// MESSAGES MAPPERS
// =============================================================================

impl TryFrom<CreateNegotiationMessageRequest> for NewNegotiationMessageDto {
    type Error = Status;

    fn try_from(proto: CreateNegotiationMessageRequest) -> Result<Self, Self::Error> {
        let id_urn = if let Some(id) = proto.id {
            Some(
                Urn::from_str(&id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid Message URN: {}", e)))?,
            )
        } else {
            None
        };

        let process_urn = Urn::from_str(&proto.negotiation_agent_process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process URN: {}", e)))?;

        let payload = if let Some(p) = proto.payload {
            prost_struct_to_serde(p)
        } else {
            serde_json::json!({})
        };

        Ok(NewNegotiationMessageDto {
            id: id_urn,
            negotiation_agent_process_id: process_urn,
            direction: proto.direction,
            protocol: proto.protocol,
            message_type: proto.message_type,
            state_transition_from: proto.state_transition_from,
            state_transition_to: proto.state_transition_to,
            payload,
        })
    }
}

impl From<NegotiationMessageDto> for NegotiationMessageResponse {
    fn from(dto: NegotiationMessageDto) -> Self {
        let inner = dto.inner;
        let payload = serde_to_prost_struct(inner.payload);

        let offer = dto.offer.map(model_offer_to_proto);
        let agreement = dto.agreement.map(model_agreement_to_proto);

        let message = ProtoMessage {
            id: inner.id,
            negotiation_agent_process_id: inner.negotiation_agent_process_id,
            direction: inner.direction,
            protocol: inner.protocol,
            message_type: inner.message_type,
            state_transition_from: inner.state_transition_from,
            state_transition_to: inner.state_transition_to,
            payload: Some(payload),
            created_at: inner.created_at.to_rfc3339(),
            offer,
            agreement,
        };

        NegotiationMessageResponse {
            message: Some(message),
        }
    }
}

// =============================================================================
// OFFERS MAPPERS
// =============================================================================

impl TryFrom<CreateOfferRequest> for NewOfferDto {
    type Error = Status;

    fn try_from(proto: CreateOfferRequest) -> Result<Self, Self::Error> {
        let id_urn = if let Some(id) = proto.id {
            Some(
                Urn::from_str(&id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid Offer URN: {}", e)))?,
            )
        } else {
            None
        };

        let process_urn = Urn::from_str(&proto.negotiation_agent_process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process URN: {}", e)))?;
        let message_urn = Urn::from_str(&proto.negotiation_agent_message_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Message URN: {}", e)))?;

        let content = if let Some(c) = proto.offer_content {
            prost_struct_to_serde(c)
        } else {
            serde_json::json!({})
        };

        Ok(NewOfferDto {
            id: id_urn,
            negotiation_agent_process_id: process_urn,
            negotiation_agent_message_id: message_urn,
            offer_id: proto.offer_id,
            offer_content: content,
        })
    }
}

impl From<OfferDto> for OfferResponse {
    fn from(dto: OfferDto) -> Self {
        OfferResponse {
            offer: Some(model_offer_to_proto(dto.inner)),
        }
    }
}

// =============================================================================
// AGREEMENTS MAPPERS
// =============================================================================

impl TryFrom<CreateAgreementRequest> for NewAgreementDto {
    type Error = Status;

    fn try_from(proto: CreateAgreementRequest) -> Result<Self, Self::Error> {
        let id_urn =
            if let Some(id) = proto.id {
                Some(Urn::from_str(&id).map_err(|e| {
                    Status::invalid_argument(format!("Invalid Agreement URN: {}", e))
                })?)
            } else {
                None
            };

        let process_urn = Urn::from_str(&proto.negotiation_agent_process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process URN: {}", e)))?;
        let message_urn = Urn::from_str(&proto.negotiation_agent_message_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Message URN: {}", e)))?;
        let target_urn = Urn::from_str(&proto.target)
            .map_err(|e| Status::invalid_argument(format!("Invalid Target URN: {}", e)))?;

        let content = if let Some(c) = proto.agreement_content {
            prost_struct_to_serde(c)
        } else {
            serde_json::json!({})
        };

        Ok(NewAgreementDto {
            id: id_urn,
            negotiation_agent_process_id: process_urn,
            negotiation_agent_message_id: message_urn,
            consumer_participant_id: proto.consumer_participant_id,
            provider_participant_id: proto.provider_participant_id,
            agreement_content: content,
            target: target_urn,
        })
    }
}

impl TryFrom<PutAgreementRequest> for EditAgreementDto {
    type Error = Status;
    fn try_from(proto: PutAgreementRequest) -> Result<Self, Self::Error> {
        Ok(EditAgreementDto { state: proto.state })
    }
}

impl From<AgreementDto> for AgreementResponse {
    fn from(dto: AgreementDto) -> Self {
        AgreementResponse {
            agreement: Some(model_agreement_to_proto(dto.inner)),
        }
    }
}

// =============================================================================
// HELPERS (Entity -> Proto)
// =============================================================================

fn model_message_to_proto(m: crate::data::entities::negotiation_message::Model) -> ProtoMessage {
    ProtoMessage {
        id: m.id,
        negotiation_agent_process_id: m.negotiation_agent_process_id,
        direction: m.direction,
        protocol: m.protocol,
        message_type: m.message_type,
        state_transition_from: m.state_transition_from,
        state_transition_to: m.state_transition_to,
        payload: Some(serde_to_prost_struct(m.payload)),
        created_at: m.created_at.to_rfc3339(),
        offer: None,
        agreement: None,
    }
}

fn model_offer_to_proto(m: crate::data::entities::offer::Model) -> ProtoOffer {
    ProtoOffer {
        id: m.id,
        negotiation_process_id: m.negotiation_agent_process_id,
        negotiation_message_id: m.negotiation_agent_message_id,
        offer_id: m.offer_id,
        offer_content: Some(serde_to_prost_struct(m.offer_content)),
        created_at: m.created_at.to_rfc3339(),
    }
}

fn model_agreement_to_proto(m: crate::data::entities::agreement::Model) -> ProtoAgreement {
    ProtoAgreement {
        id: m.id,
        negotiation_agent_process_id: m.negotiation_agent_process_id,
        negotiation_agent_message_id: m.negotiation_agent_message_id,
        consumer_participant_id: m.consumer_participant_id,
        provider_participant_id: m.provider_participant_id,
        agreement_content: Some(serde_to_prost_struct(m.agreement_content)),
        target: m.target,
        state: m.state,
        created_at: m.created_at.to_rfc3339(),
        updated_at: m.updated_at.map(|d| d.to_rfc3339()),
    }
}

// =============================================================================
// JSON HELPERS
// =============================================================================

/// Convierte un serde_json::Value a prost_types::Struct
pub fn serde_to_prost_struct(json: JsonValue) -> Struct {
    match json {
        JsonValue::Object(map) => {
            let fields = map
                .into_iter()
                .map(|(k, v)| (k, serde_to_prost_value(v)))
                .collect();
            Struct { fields }
        }
        _ => Struct::default(), // Si no es un objeto, devolvemos struct vacío
    }
}

/// Convierte un prost_types::Struct a serde_json::Value
pub fn prost_struct_to_serde(proto_struct: Struct) -> JsonValue {
    let map: Map<String, JsonValue> = proto_struct
        .fields
        .into_iter()
        .map(|(k, v)| (k, prost_value_to_serde(v)))
        .collect();
    JsonValue::Object(map)
}

// --- Funciones Auxiliares Recursivas ---

fn serde_to_prost_value(json: JsonValue) -> ProstValue {
    let kind = match json {
        JsonValue::Null => Kind::NullValue(0),
        JsonValue::Bool(v) => Kind::BoolValue(v),
        JsonValue::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        JsonValue::String(s) => Kind::StringValue(s),
        JsonValue::Array(v) => Kind::ListValue(ListValue {
            values: v.into_iter().map(serde_to_prost_value).collect(),
        }),
        JsonValue::Object(v) => Kind::StructValue(Struct {
            fields: v
                .into_iter()
                .map(|(k, v)| (k, serde_to_prost_value(v)))
                .collect(),
        }),
    };
    ProstValue { kind: Some(kind) }
}

fn prost_value_to_serde(prost: ProstValue) -> JsonValue {
    match prost.kind {
        Some(Kind::NullValue(_)) | None => JsonValue::Null,
        Some(Kind::BoolValue(v)) => JsonValue::Bool(v),
        Some(Kind::NumberValue(n)) => {
            // serde_json::Number::from_f64 puede fallar si es infinito/NaN,
            // fallback a Null o 0 si ocurre.
            serde_json::Number::from_f64(n)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)
        }
        Some(Kind::StringValue(s)) => JsonValue::String(s),
        Some(Kind::ListValue(l)) => {
            JsonValue::Array(l.values.into_iter().map(prost_value_to_serde).collect())
        }
        Some(Kind::StructValue(s)) => {
            let map = s
                .fields
                .into_iter()
                .map(|(k, v)| (k, prost_value_to_serde(v)))
                .collect();
            JsonValue::Object(map)
        }
    }
}
