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

use crate::entities::transfer_process::{
    EditTransferProcessDto, NewTransferProcessDto, TransferProcessDto,
};
use crate::grpc::api::transfer_messages::TransferMessageResponse;
use crate::grpc::api::transfer_processes::{
    BatchProcessRequest, CreateProcessRequest, TransferProcessResponse, UpdateProcessRequest,
};
use chrono::DateTime;
use common::batch_requests::BatchRequests;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use tonic::Status;
use urn::Urn;

// -----------------------------------------------------------------------------
// PROTO -> DOMAIN (TryFrom)
// -----------------------------------------------------------------------------

impl TryFrom<CreateProcessRequest> for NewTransferProcessDto {
    type Error = Status;

    fn try_from(proto: CreateProcessRequest) -> Result<Self, Self::Error> {
        // 1. URNs
        let agreement_urn = Urn::from_str(&proto.agreement_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Agreement URN: {}", e)))?;

        let id_urn = if let Some(id) = proto.id {
            Some(
                Urn::from_str(&id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid Process URN: {}", e)))?,
            )
        } else {
            None
        };

        // 2. JSON Properties
        let properties =
            if let Some(json_str) = proto.properties_json {
                Some(serde_json::from_str(&json_str).map_err(|e| {
                    Status::invalid_argument(format!("Invalid properties JSON: {}", e))
                })?)
            } else {
                None
            };

        // 3. Identifiers (Map<String, String> -> HashMap<String, String>)
        let identifiers = if proto.identifiers.is_empty() {
            None
        } else {
            Some(proto.identifiers) // Rust HashMap es compatible
        };

        Ok(NewTransferProcessDto {
            id: id_urn,
            state: proto.state,
            associated_agent_peer: proto.associated_agent_peer,
            protocol: proto.protocol,
            connector_instance_id: proto.connector_instance_id,
            transfer_direction: proto.transfer_direction,
            agreement_id: agreement_urn,
            callback_address: None,
            role: "".to_string(),
            state_attribute: proto.state_attribute,
            properties,
            identifiers,
        })
    }
}

impl TryFrom<UpdateProcessRequest> for EditTransferProcessDto {
    type Error = Status;

    fn try_from(proto: UpdateProcessRequest) -> Result<Self, Self::Error> {
        // Parsing optional json
        let properties = parse_optional_json(proto.properties_json)?;
        let error_details = parse_optional_json(proto.error_details_json)?;

        let identifiers = if proto.identifiers.is_empty() {
            None
        } else {
            Some(proto.identifiers)
        };

        Ok(EditTransferProcessDto {
            state: proto.state,
            state_attribute: proto.state_attribute,
            properties,
            error_details,
            identifiers,
        })
    }
}

impl From<BatchProcessRequest> for BatchRequests {
    fn from(proto: BatchProcessRequest) -> Self {
        BatchRequests {
            ids: proto
                .ids
                .iter()
                .map(|i| Urn::from_str(i).unwrap())
                .collect(),
        }
    }
}

// -----------------------------------------------------------------------------
// DOMAIN -> PROTO (From)
// -----------------------------------------------------------------------------

impl From<TransferProcessDto> for TransferProcessResponse {
    fn from(dto: TransferProcessDto) -> Self {
        let model = dto.inner;

        let properties_json = serde_json::to_string(&model.properties).unwrap_or_default();
        let error_details_json = model
            .error_details
            .map(|j| serde_json::to_string(&j).unwrap_or_default());

        // 2. dates
        let created_at = to_prost_timestamp(DateTime::from(model.created_at));
        let updated_at = model
            .updated_at
            .map(|d| to_prost_timestamp(DateTime::from(d)));

        // 3. nested
        let messages_proto: Vec<TransferMessageResponse> = dto
            .messages
            .into_iter()
            .map(|msg_model| {
                let msg_dto =
                    crate::entities::transfer_messages::TransferMessageDto { inner: msg_model };
                msg_dto.into()
            })
            .collect();

        Self {
            id: model.id,
            state: model.state,
            state_attribute: model.state_attribute,
            associated_agent_peer: model.associated_agent_peer,
            protocol: model.protocol,
            transfer_direction: model.transfer_direction,
            agreement_id: model.agreement_id,
            callback_address: model.callback_address,
            role: model.role,

            properties_json,
            error_details_json,

            created_at: created_at.into(),
            updated_at,

            identifiers: dto.identifiers,
            messages: messages_proto,
        }
    }
}

// Helpers
fn parse_optional_json(input: Option<String>) -> Result<Option<JsonValue>, Status> {
    match input {
        Some(s) if !s.trim().is_empty() => serde_json::from_str(&s)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON: {}", e)))
            .map(Some),
        _ => Ok(None),
    }
}

fn to_prost_timestamp(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
