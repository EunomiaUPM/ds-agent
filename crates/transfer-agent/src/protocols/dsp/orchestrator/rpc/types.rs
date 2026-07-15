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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferCompletionMessageDto, TransferErrorDto, TransferProcessAckDto,
    TransferProcessMessageType, TransferProcessMessageWrapper, TransferRequestMessageDto,
    TransferStartMessageDto, TransferSuspensionMessageDto, TransferTerminationMessageDto,
};
use common::dsp_common::context_field::ContextField;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;
use urn::Urn;

// Trait ────────────────────────────────────────────────────────────────────

/// Uniform accessor interface over every inbound RPC message DTO.
///
/// Each concrete DTO (request, start, suspension, …) only carries the fields
/// that make sense for its lifecycle step.  All other accessors return `None`,
/// so callers can use the trait without knowing the concrete type.
#[allow(dead_code)]
pub trait RpcTransferProcessMessageTrait: Debug + Send + Sync {
    fn get_consumer_pid(&self) -> Option<Urn>;
    fn get_provider_pid(&self) -> Option<Urn>;
    fn get_associated_agent_peer(&self) -> Option<String>;
    fn get_agreement_id(&self) -> Option<Urn>;
    fn get_format(&self) -> Option<String>;
    fn get_data_address(&self) -> Option<DataAddressDto>;
    fn get_provider_address(&self) -> Option<String>;
    fn get_callback_address(&self) -> Option<String>;
    fn get_error_code(&self) -> Option<String>;
    fn get_error_reason(&self) -> Option<Vec<String>>;
    fn get_message(&self) -> TransferProcessMessageType;
}

// TransferRequestMessage ───────────────────────────────────────────────────

/// RPC payload for initiating a new transfer process (Consumer - Provider).
///
/// Carries everything needed to build a DSP `TransferRequestMessage` plus the
/// routing fields (`associated_agent_peer`, `provider_address`) that are
/// consumed locally and never forwarded on the wire.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferRequestMessageDto {
    pub associated_agent_peer: String,
    pub agreement_id: Urn,
    pub format: String,
    pub data_address: Option<DataAddressDto>,
    pub provider_address: String,
    pub callback_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,
}

impl Into<TransferProcessMessageWrapper<TransferRequestMessageDto>>
    for RpcTransferRequestMessageDto
{
    fn into(self) -> TransferProcessMessageWrapper<TransferRequestMessageDto> {
        let consumer_pid = format!("urn:consumer-pid:{}", uuid::Uuid::new_v4());
        let consumer_pid_urn = Urn::from_str(consumer_pid.as_str()).unwrap();
        let callback_address = self.callback_address.clone();
        TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: TransferProcessMessageType::TransferRequestMessage,
            dto: TransferRequestMessageDto {
                agreement_id: self.agreement_id,
                format: self.format,
                data_address: self.data_address,
                callback_address,
                consumer_pid: consumer_pid_urn,
                auto_start: self.auto_start,
            },
        }
    }
}

impl RpcTransferProcessMessageTrait for RpcTransferRequestMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        None
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        None
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        Some(self.associated_agent_peer.clone())
    }

    fn get_agreement_id(&self) -> Option<Urn> {
        Some(self.agreement_id.clone())
    }

    fn get_format(&self) -> Option<String> {
        Some(self.format.clone())
    }

    fn get_data_address(&self) -> Option<DataAddressDto> {
        self.data_address.clone()
    }

    fn get_provider_address(&self) -> Option<String> {
        Some(self.provider_address.clone())
    }

    fn get_callback_address(&self) -> Option<String> {
        Some(self.callback_address.clone())
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_message(&self) -> TransferProcessMessageType {
        TransferProcessMessageType::TransferRequestMessage
    }
}

// TransferStartMessage ─────────────────────────────────────────────────────

/// RPC payload for starting an already-requested transfer process.
///
/// Both PIDs must be known at this point: `consumer_pid` was generated when
/// the request was first submitted; `provider_pid` was received in the
/// provider's acknowledgement.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferStartMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
}

impl Into<TransferProcessMessageWrapper<TransferStartMessageDto>> for RpcTransferStartMessageDto {
    fn into(self) -> TransferProcessMessageWrapper<TransferStartMessageDto> {
        let consumer_pid = self.get_consumer_pid().unwrap();
        let provider_pid = self.get_provider_pid().unwrap();
        TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: TransferProcessMessageType::TransferStartMessage,
            dto: TransferStartMessageDto {
                data_address: None,
                provider_pid,
                consumer_pid,
            },
        }
    }
}

impl RpcTransferProcessMessageTrait for RpcTransferStartMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        None
    }

    fn get_agreement_id(&self) -> Option<Urn> {
        None
    }

    fn get_format(&self) -> Option<String> {
        None
    }

    fn get_data_address(&self) -> Option<DataAddressDto> {
        None
    }

    fn get_provider_address(&self) -> Option<String> {
        None
    }

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_message(&self) -> TransferProcessMessageType {
        TransferProcessMessageType::TransferStartMessage
    }
}

// TransferSuspensionMessage ────────────────────────────────────────────────

/// RPC payload for temporarily suspending an active transfer.
///
/// `code` and `reason` are optional; when supplied they are forwarded verbatim
/// in the DSP `TransferSuspensionMessage`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferSuspensionMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
}

impl Into<TransferProcessMessageWrapper<TransferSuspensionMessageDto>>
    for RpcTransferSuspensionMessageDto
{
    fn into(self) -> TransferProcessMessageWrapper<TransferSuspensionMessageDto> {
        let consumer_pid = self.get_consumer_pid().unwrap();
        let provider_pid = self.get_provider_pid().unwrap();
        TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: TransferProcessMessageType::TransferSuspensionMessage,
            dto: TransferSuspensionMessageDto {
                provider_pid,
                consumer_pid,
                code: self.get_error_code(),
                reason: self.get_error_reason(),
            },
        }
    }
}

impl RpcTransferProcessMessageTrait for RpcTransferSuspensionMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        None
    }

    fn get_agreement_id(&self) -> Option<Urn> {
        None
    }

    fn get_format(&self) -> Option<String> {
        None
    }

    fn get_data_address(&self) -> Option<DataAddressDto> {
        None
    }

    fn get_provider_address(&self) -> Option<String> {
        None
    }

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        self.code.clone()
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        self.reason.clone()
    }

    fn get_message(&self) -> TransferProcessMessageType {
        TransferProcessMessageType::TransferSuspensionMessage
    }
}

// TransferCompletionMessage ────────────────────────────────────────────────

/// RPC payload for marking a transfer as successfully completed.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferCompletionMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
}

impl Into<TransferProcessMessageWrapper<TransferCompletionMessageDto>>
    for RpcTransferCompletionMessageDto
{
    fn into(self) -> TransferProcessMessageWrapper<TransferCompletionMessageDto> {
        let consumer_pid = self.get_consumer_pid().unwrap();
        let provider_pid = self.get_provider_pid().unwrap();
        TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: TransferProcessMessageType::TransferCompletionMessage,
            dto: TransferCompletionMessageDto {
                provider_pid,
                consumer_pid,
            },
        }
    }
}

impl RpcTransferProcessMessageTrait for RpcTransferCompletionMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        None
    }

    fn get_agreement_id(&self) -> Option<Urn> {
        None
    }

    fn get_format(&self) -> Option<String> {
        None
    }

    fn get_data_address(&self) -> Option<DataAddressDto> {
        None
    }

    fn get_provider_address(&self) -> Option<String> {
        None
    }

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_message(&self) -> TransferProcessMessageType {
        TransferProcessMessageType::TransferCompletionMessage
    }
}

// TransferTerminationMessage ───────────────────────────────────────────────

/// RPC payload for permanently terminating a transfer (abnormal end).
///
/// Like suspension, `code` and `reason` are optional diagnostic fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferTerminationMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
}

impl Into<TransferProcessMessageWrapper<TransferTerminationMessageDto>>
    for RpcTransferTerminationMessageDto
{
    fn into(self) -> TransferProcessMessageWrapper<TransferTerminationMessageDto> {
        let consumer_pid = self.get_consumer_pid().unwrap();
        let provider_pid = self.get_provider_pid().unwrap();
        TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: TransferProcessMessageType::TransferTerminationMessage,
            dto: TransferTerminationMessageDto {
                provider_pid,
                consumer_pid,
                code: self.get_error_code(),
                reason: self.get_error_reason(),
            },
        }
    }
}

impl RpcTransferProcessMessageTrait for RpcTransferTerminationMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        None
    }

    fn get_agreement_id(&self) -> Option<Urn> {
        None
    }

    fn get_format(&self) -> Option<String> {
        None
    }

    fn get_data_address(&self) -> Option<DataAddressDto> {
        None
    }
    fn get_provider_address(&self) -> Option<String> {
        None
    }

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        self.code.clone()
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        self.reason.clone()
    }

    fn get_message(&self) -> TransferProcessMessageType {
        TransferProcessMessageType::TransferTerminationMessage
    }
}

// Response wrappers ────────────────────────────────────────────────────────

/// Successful RPC response: echoes the original request, the DSP acknowledgement,
/// and the updated transfer process model.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferMessageDto<T> {
    pub request: T,
    pub response: TransferProcessMessageWrapper<TransferProcessAckDto>,
    pub transfer_agent_model: TransferProcessDto,
}

/// Error RPC response: echoes the original request alongside the DSP error body.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcTransferErrorDto<T> {
    pub request: T,
    pub error: TransferProcessMessageWrapper<TransferErrorDto>,
}
