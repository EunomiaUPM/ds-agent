/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationErrorMessageDto,
    NegotiationEventMessageDto, NegotiationEventType, NegotiationOfferInitMessageDto,
    NegotiationOfferMessageDto, NegotiationProcessMessageType, NegotiationProcessMessageWrapper,
    NegotiationRequestInitMessageDto, NegotiationRequestMessageDto,
    NegotiationTerminationMessageDto, NegotiationVerificationMessageDto,
};
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::{ContractRequestMessageOfferTypes, OdrlAgreement};
use common::utils::get_urn;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::str::FromStr;
use urn::Urn;

pub trait RpcNegotiationProcessMessageTrait: Debug + Send + Sync {
    fn get_consumer_pid(&self) -> Option<Urn>;
    fn get_provider_pid(&self) -> Option<Urn>;
    fn get_associated_agent_peer(&self) -> Option<String>;
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes>;
    fn get_agreement(&self) -> Option<OdrlAgreement>;
    fn get_provider_address(&self) -> Option<String>;
    fn get_callback_address(&self) -> Option<String>;
    fn get_event_type(&self) -> Option<NegotiationEventType>;
    fn get_error_code(&self) -> Option<String>;
    fn get_error_reason(&self) -> Option<Vec<String>>;
    fn get_message(&self) -> NegotiationProcessMessageType;
    fn as_json(&self) -> Value;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationRequestInitMessageDto {
    associated_agent_peer: String,
    provider_address: String,
    callback_address: String,
    offer: ContractRequestMessageOfferTypes,
}

impl RpcNegotiationRequestInitMessageDto {
    pub fn new(
        associated_agent_peer: String,
        provider_address: String,
        callback_address: String,
        offer: ContractRequestMessageOfferTypes,
    ) -> Self {
        Self {
            associated_agent_peer,
            provider_address,
            callback_address,
            offer,
        }
    }
}

impl Into<NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>>
    for RpcNegotiationRequestInitMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto> {
        let consumer_pid = format!("urn:consumer-pid:{}", uuid::Uuid::new_v4());
        let consumer_pid_urn = Urn::from_str(consumer_pid.as_str()).unwrap();
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationRequestMessage,
            dto: NegotiationRequestInitMessageDto {
                callback_address: Some(self.callback_address),
                consumer_pid: consumer_pid_urn,
                offer: self.offer,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationRequestInitMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        None
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        None
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        Some(self.associated_agent_peer.clone())
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationRequestMessage
    }
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        Some(self.offer.clone())
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationRequestMessageDto {
    offer: ContractRequestMessageOfferTypes,
    provider_pid: Urn,
    consumer_pid: Urn,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>>
    for RpcNegotiationRequestMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationRequestMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationRequestMessage,
            dto: NegotiationRequestMessageDto {
                consumer_pid: self.consumer_pid,
                provider_pid: self.provider_pid,
                offer: self.offer,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationRequestMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationRequestMessage
    }
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        Some(self.offer.clone())
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationOfferInitMessageDto {
    associated_agent_peer: String,
    provider_address: String,
    callback_address: String,
    offer: ContractRequestMessageOfferTypes,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>>
    for RpcNegotiationOfferInitMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto> {
        let provider_pid = format!("urn:provider-pid:{}", uuid::Uuid::new_v4());
        let provider_pid_urn = Urn::from_str(provider_pid.as_str()).unwrap();
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationOfferMessage,
            dto: NegotiationOfferInitMessageDto {
                provider_pid: provider_pid_urn,
                offer: self.offer,
                callback_address: Some(self.callback_address),
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationOfferInitMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        None
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        None
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
        Some(self.associated_agent_peer.clone())
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationOfferMessage
    }
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        Some(self.offer.clone())
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationOfferMessageDto {
    offer: ContractRequestMessageOfferTypes,
    provider_pid: Urn,
    consumer_pid: Urn,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>>
    for RpcNegotiationOfferMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationOfferMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationOfferMessage,
            dto: NegotiationOfferMessageDto {
                provider_pid: self.provider_pid,
                offer: self.offer,
                consumer_pid: self.consumer_pid,
                callback_address: None,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationOfferMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationOfferMessage
    }

    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        Some(self.offer.clone())
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationAgreementMessageDto {
    provider_pid: Urn,
    consumer_pid: Urn,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>>
    for RpcNegotiationAgreementMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationAgreementMessage,
            dto: NegotiationAgreementMessageDto {
                consumer_pid: self.consumer_pid,
                provider_pid: self.provider_pid,
                agreement: OdrlAgreement::default(),
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationAgreementMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationAgreementMessage
    }

    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationVerificationMessageDto {
    provider_pid: Urn,
    consumer_pid: Urn,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>>
    for RpcNegotiationVerificationMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationAgreementVerificationMessage,
            dto: NegotiationVerificationMessageDto {
                consumer_pid: self.consumer_pid,
                provider_pid: self.provider_pid,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationVerificationMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationAgreementVerificationMessage
    }

    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationEventAcceptedMessageDto {
    provider_pid: Urn,
    consumer_pid: Urn,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationEventMessageDto>>
    for RpcNegotiationEventAcceptedMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationEventMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationEventMessage(
                NegotiationEventType::ACCEPTED,
            ),
            dto: NegotiationEventMessageDto {
                consumer_pid: self.consumer_pid,
                provider_pid: self.provider_pid,
                event_type: NegotiationEventType::ACCEPTED,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationEventAcceptedMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationEventMessage(NegotiationEventType::ACCEPTED)
    }
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }
    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        Some(NegotiationEventType::ACCEPTED)
    }

    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationEventFinalizedMessageDto {
    provider_pid: Urn,
    consumer_pid: Urn,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationEventMessageDto>>
    for RpcNegotiationEventFinalizedMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationEventMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationEventMessage(
                NegotiationEventType::FINALIZED,
            ),
            dto: NegotiationEventMessageDto {
                consumer_pid: self.consumer_pid,
                provider_pid: self.provider_pid,
                event_type: NegotiationEventType::FINALIZED,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationEventFinalizedMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationEventMessage(NegotiationEventType::FINALIZED)
    }
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }
    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        Some(NegotiationEventType::FINALIZED)
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationTerminationMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
}

impl Into<NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>>
    for RpcNegotiationTerminationMessageDto
{
    fn into(self) -> NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto> {
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationTerminationMessage,
            dto: NegotiationTerminationMessageDto {
                consumer_pid: self.consumer_pid,
                provider_pid: self.provider_pid,
                code: self.code,
                reason: self.reason,
            },
        }
    }
}

impl RpcNegotiationProcessMessageTrait for RpcNegotiationTerminationMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_associated_agent_peer(&self) -> Option<String> {
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

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationTerminationMessage
    }

    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        None
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationMessageDto<T> {
    pub request: T,
    pub response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
    pub negotiation_agent_model: NegotiationProcessDto,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcNegotiationErrorDto<T> {
    pub request: T,
    pub error: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto>,
}
