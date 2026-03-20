/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::entities::transfer_messages::{NewTransferMessageDto, TransferMessageDto};
use crate::grpc::api::transfer_messages::{
    CreateMessageRequest, PaginationRequestMessages, TransferMessageResponse,
};
use crate::http::transfer_messages::PaginationParams;
use chrono::DateTime;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use tonic::Status;
use urn::Urn;
impl TryFrom<CreateMessageRequest> for NewTransferMessageDto {
    type Error = Status;

    fn try_from(proto: CreateMessageRequest) -> Result<Self, Self::Error> {
        let process_urn = Urn::from_str(&proto.transfer_agent_process_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid Process URN: {}", e)))?;
        let id_urn = if let Some(id_str) = proto.id {
            Some(
                Urn::from_str(&id_str)
                    .map_err(|e| Status::invalid_argument(format!("Invalid Message URN: {}", e)))?,
            )
        } else {
            None
        };
        let payload: Option<JsonValue> = if let Some(json_str) = proto.payload_json {
            if json_str.trim().is_empty() {
                None
            } else {
                Some(serde_json::from_str(&json_str).map_err(|e| {
                    Status::invalid_argument(format!("Invalid JSON payload: {}", e))
                })?)
            }
        } else {
            None
        };

        Ok(NewTransferMessageDto {
            id: id_urn,
            transfer_agent_process_id: process_urn,
            direction: proto.direction,
            protocol: proto.protocol,
            message_type: proto.message_type,
            state_transition_from: proto.state_transition_from,
            state_transition_to: proto.state_transition_to,
            payload,
        })
    }
}

impl From<PaginationRequestMessages> for PaginationParams {
    fn from(proto: PaginationRequestMessages) -> Self {
        Self { limit: proto.limit, page: proto.page }
    }
}

impl From<TransferMessageDto> for TransferMessageResponse {
    fn from(dto: TransferMessageDto) -> Self {
        let model = dto.inner;
        let payload_json =
            model.payload.map(|json| serde_json::to_string(&json).unwrap_or_default());
        let created_at = to_prost_timestamp(DateTime::from(model.created_at));

        Self {
            id: model.id.to_string(),
            transfer_agent_process_id: model.transfer_agent_process_id.to_string(),
            direction: model.direction,
            protocol: model.protocol,
            state_transition_from: model.state_transition_from,
            message_type: model.message_type,
            state_transition_to: model.state_transition_to,
            payload_json,
            created_at: Some(created_at),
            updated_at: None,
        }
    }
}

fn to_prost_timestamp(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp { seconds: dt.timestamp(), nanos: dt.timestamp_subsec_nanos() as i32 }
}
