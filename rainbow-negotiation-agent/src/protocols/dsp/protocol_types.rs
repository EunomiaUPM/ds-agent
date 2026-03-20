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

use crate::entities::negotiation_process::NegotiationProcessDto;
use rainbow_common::dsp_common::context_field::ContextField;
use rainbow_common::dsp_common::odrl::{ContractRequestMessageOfferTypes, OdrlAgreement};
use serde::{Deserialize, Serialize};
use ymir::errors::Errors;
use serde_json::Value;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use tracing::error;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationProcessMessageWrapper<T>
where
    T: NegotiationProcessMessageTrait,
{
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub _type: NegotiationProcessMessageType,
    #[serde(flatten)]
    pub dto: T,
}

pub trait NegotiationProcessMessageTrait: Debug + Send + Sync {
    fn get_consumer_pid(&self) -> Option<Urn>;
    fn get_provider_pid(&self) -> Option<Urn>;
    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes>;
    fn get_agreement(&self) -> Option<OdrlAgreement>;
    fn get_event_type(&self) -> Option<NegotiationEventType>;
    fn get_callback_address(&self) -> Option<String>;
    fn get_error_code(&self) -> Option<String>;
    fn get_error_reason(&self) -> Option<Vec<String>>;
    fn get_state(&self) -> Option<NegotiationProcessState>;
    fn get_message(&self) -> NegotiationProcessMessageType;
    fn as_json(&self) -> Value;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationRequestInitMessageDto {
    pub consumer_pid: Urn,
    pub offer: ContractRequestMessageOfferTypes,
    pub callback_address: Option<String>,
}

impl NegotiationProcessMessageTrait for NegotiationRequestInitMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        None
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

    fn get_callback_address(&self) -> Option<String> {
        self.callback_address.clone()
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationRequestMessage
    }

    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationRequestMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub offer: ContractRequestMessageOfferTypes,
}

impl NegotiationProcessMessageTrait for NegotiationRequestMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
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

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationRequestMessage
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationOfferInitMessageDto {
    pub provider_pid: Urn,
    pub offer: ContractRequestMessageOfferTypes,
    pub callback_address: Option<String>,
}

impl NegotiationProcessMessageTrait for NegotiationOfferInitMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        None
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
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

    fn get_callback_address(&self) -> Option<String> {
        self.callback_address.clone()
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationOfferMessage
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationOfferMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub offer: ContractRequestMessageOfferTypes,
    pub callback_address: Option<String>,
}

impl NegotiationProcessMessageTrait for NegotiationOfferMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
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

    fn get_callback_address(&self) -> Option<String> {
        self.callback_address.clone()
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationOfferMessage
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationAgreementMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub agreement: OdrlAgreement,
}

impl NegotiationProcessMessageTrait for NegotiationAgreementMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        Some(self.agreement.clone())
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
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

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationAgreementMessage
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationVerificationMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
}

impl NegotiationProcessMessageTrait for NegotiationVerificationMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
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

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationAgreementVerificationMessage
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationEventMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub event_type: NegotiationEventType,
}

impl NegotiationProcessMessageTrait for NegotiationEventMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
    }

    fn get_offer(&self) -> Option<ContractRequestMessageOfferTypes> {
        None
    }

    fn get_agreement(&self) -> Option<OdrlAgreement> {
        None
    }

    fn get_event_type(&self) -> Option<NegotiationEventType> {
        Some(self.event_type.clone())
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

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationEventMessage(self.event_type.clone())
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationTerminationMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
}

impl NegotiationProcessMessageTrait for NegotiationTerminationMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
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

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        self.code.clone()
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        self.reason.clone()
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationTerminationMessage
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationAckMessageDto {
    pub consumer_pid: Urn,
    pub provider_pid: Urn,
    pub state: NegotiationProcessState,
}

impl NegotiationProcessMessageTrait for NegotiationAckMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        Some(self.consumer_pid.clone())
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        Some(self.provider_pid.clone())
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

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        None
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        None
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        Some(self.state.clone())
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationProcess
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NegotiationErrorMessageDto {
    pub consumer_pid: Option<Urn>,
    pub provider_pid: Option<Urn>,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
}

impl NegotiationProcessMessageTrait for NegotiationErrorMessageDto {
    fn get_consumer_pid(&self) -> Option<Urn> {
        self.consumer_pid.clone()
    }

    fn get_provider_pid(&self) -> Option<Urn> {
        self.provider_pid.clone()
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

    fn get_callback_address(&self) -> Option<String> {
        None
    }

    fn get_error_code(&self) -> Option<String> {
        self.code.clone()
    }

    fn get_error_reason(&self) -> Option<Vec<String>> {
        self.reason.clone()
    }

    fn get_state(&self) -> Option<NegotiationProcessState> {
        None
    }

    fn get_message(&self) -> NegotiationProcessMessageType {
        NegotiationProcessMessageType::NegotiationError
    }
    fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum NegotiationProcessState {
    Requested,
    Offered,
    Accepted,
    Agreed,
    Verified,
    Finalized,
    Terminated,
}
impl Display for NegotiationProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            NegotiationProcessState::Requested => "REQUESTED".to_string(),
            NegotiationProcessState::Offered => "OFFERED".to_string(),
            NegotiationProcessState::Accepted => "ACCEPTED".to_string(),
            NegotiationProcessState::Agreed => "AGREED".to_string(),
            NegotiationProcessState::Verified => "VERIFIED".to_string(),
            NegotiationProcessState::Finalized => "FINALIZED".to_string(),
            NegotiationProcessState::Terminated => "TERMINATED".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for NegotiationProcessState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REQUESTED" => Ok(NegotiationProcessState::Requested),
            "OFFERED" => Ok(NegotiationProcessState::Offered),
            "ACCEPTED" => Ok(NegotiationProcessState::Accepted),
            "AGREED" => Ok(NegotiationProcessState::Agreed),
            "VERIFIED" => Ok(NegotiationProcessState::Verified),
            "FINALIZED" => Ok(NegotiationProcessState::Finalized),
            "TERMINATED" => Ok(NegotiationProcessState::Terminated),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum NegotiationEventType {
    ACCEPTED,
    FINALIZED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum NegotiationProcessMessageType {
    #[serde(rename = "ContractRequestMessage")]
    NegotiationRequestMessage,
    #[serde(rename = "ContractOfferMessage")]
    NegotiationOfferMessage,
    #[serde(rename = "ContractNegotiationEventMessage")]
    NegotiationEventMessage(NegotiationEventType),
    #[serde(rename = "ContractAgreementMessage")]
    NegotiationAgreementMessage,
    #[serde(rename = "ContractAgreementVerificationMessage")]
    NegotiationAgreementVerificationMessage,
    #[serde(rename = "ContractNegotiationTerminationMessage")]
    NegotiationTerminationMessage,
    #[serde(rename = "ContractNegotiation")]
    NegotiationProcess,
    #[serde(rename = "ContractNegotiationError")]
    NegotiationError,
}

impl Display for NegotiationProcessMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            NegotiationProcessMessageType::NegotiationRequestMessage => {
                "ContractRequestMessage".to_string()
            }
            NegotiationProcessMessageType::NegotiationOfferMessage => {
                "ContractOfferMessage".to_string()
            }
            NegotiationProcessMessageType::NegotiationEventMessage(_) => {
                "ContractNegotiationEventMessage".to_string()
            }
            NegotiationProcessMessageType::NegotiationAgreementMessage => {
                "ContractAgreementMessage".to_string()
            }
            NegotiationProcessMessageType::NegotiationAgreementVerificationMessage => {
                "ContractAgreementVerificationMessage".to_string()
            }
            NegotiationProcessMessageType::NegotiationTerminationMessage => {
                "ContractNegotiationTerminationMessage".to_string()
            }
            NegotiationProcessMessageType::NegotiationProcess => "ContractNegotiation".to_string(),
            NegotiationProcessMessageType::NegotiationError => {
                "ContractNegotiationError".to_string()
            }
        };
        write!(f, "{}", str)
    }
}

impl FromStr for NegotiationProcessMessageType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ContractRequestMessage" => {
                Ok(NegotiationProcessMessageType::NegotiationRequestMessage)
            }
            "ContractOfferMessage" => Ok(NegotiationProcessMessageType::NegotiationOfferMessage),
            "ContractNegotiationEventMessage" => {
                Ok(NegotiationProcessMessageType::NegotiationEventMessage(
                    NegotiationEventType::ACCEPTED,
                ))
            }
            "ContractAgreementMessage" => {
                Ok(NegotiationProcessMessageType::NegotiationAgreementMessage)
            }
            "ContractAgreementVerificationMessage" => {
                Ok(NegotiationProcessMessageType::NegotiationAgreementVerificationMessage)
            }
            "ContractNegotiationTerminationMessage" => {
                Ok(NegotiationProcessMessageType::NegotiationTerminationMessage)
            }
            "ContractNegotiation" => Ok(NegotiationProcessMessageType::NegotiationProcess),
            "ContractNegotiationError" => Ok(NegotiationProcessMessageType::NegotiationError),
            _ => Err(Errors::parse("Invalid negotiation message", None)),
        }
    }
}

impl From<NegotiationProcessMessageType> for NegotiationProcessState {
    fn from(value: NegotiationProcessMessageType) -> Self {
        match value {
            NegotiationProcessMessageType::NegotiationRequestMessage => {
                NegotiationProcessState::Requested
            }
            NegotiationProcessMessageType::NegotiationOfferMessage => {
                NegotiationProcessState::Offered
            }
            NegotiationProcessMessageType::NegotiationEventMessage(ev) => match ev {
                NegotiationEventType::ACCEPTED => NegotiationProcessState::Accepted,
                NegotiationEventType::FINALIZED => NegotiationProcessState::Finalized,
            },
            NegotiationProcessMessageType::NegotiationAgreementMessage => {
                NegotiationProcessState::Agreed
            }
            NegotiationProcessMessageType::NegotiationAgreementVerificationMessage => {
                NegotiationProcessState::Verified
            }
            NegotiationProcessMessageType::NegotiationTerminationMessage => {
                NegotiationProcessState::Terminated
            }
            NegotiationProcessMessageType::NegotiationProcess => {
                NegotiationProcessState::Terminated
            }
            NegotiationProcessMessageType::NegotiationError => NegotiationProcessState::Terminated,
        }
    }
}

impl TryFrom<NegotiationProcessDto> for NegotiationProcessMessageWrapper<NegotiationAckMessageDto> {
    type Error = Errors;

    fn try_from(value: NegotiationProcessDto) -> Result<Self, Self::Error> {
        let consumer_str = match value.identifiers.get("consumerPid") {
            Some(val) => val,
            None => {
                let err = Errors::parse(
                    "Missing 'consumerPid' in NegotiationProcessDto identifiers map",
                    None,
                );
                error!("{}", err);
                return Err(err);
            }
        };
        let consumer_pid = match Urn::from_str(consumer_str) {
            Ok(urn) => urn,
            Err(e) => {
                let err = Errors::parse(
                    format!("Invalid URN format for consumerPid '{}': {}", consumer_str, e),
                    None,
                );
                error!("{}", err);
                return Err(err);
            }
        };

        let provider_str = match value.identifiers.get("providerPid") {
            Some(val) => val,
            None => {
                let err = Errors::parse(
                    "Missing 'providerPid' in NegotiationProcessDto identifiers map",
                    None,
                );
                error!("{}", err);
                return Err(err);
            }
        };
        let provider_pid = match Urn::from_str(provider_str) {
            Ok(urn) => urn,
            Err(e) => {
                let err = Errors::parse(
                    format!("Invalid URN format for providerPid '{}': {}", provider_str, e),
                    None,
                );
                error!("{}", err);
                return Err(err);
            }
        };

        let state = match value.inner.state.parse::<NegotiationProcessState>() {
            Ok(s) => s,
            Err(_) => {
                let err = Errors::parse(
                    format!(
                        "Invalid or unknown NegotiationProcessState '{}' in database model",
                        value.inner.state
                    ),
                    None,
                );
                error!("{}", err);
                return Err(err);
            }
        };

        Ok(Self {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationProcess,
            dto: NegotiationAckMessageDto { consumer_pid, provider_pid, state },
        })
    }
}
