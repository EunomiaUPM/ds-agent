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


mod consumer_pull;
mod consumer_push;
pub mod local;
mod provider_pull;
mod provider_push;
mod strategy;

use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::context_common::TransferContextConnectorRole;
use crate::protocols::dsp::entities::data_address::{DataAddressDto, EndpointPropertyDto};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use common::dsp_common::data_address::DataAddress;
use ymir::oauth::Owner;
use dataplane::{DataplaneAddress, DataplaneContinuation};
use urn::Urn;
use ymir::errors::Outcome;

/// What the dataplane hooks need about a transfer, whichever side and direction drives it.
#[derive(Debug)]
pub struct DataplaneHookContext {
    pub process_id: Urn,
    pub owner: Owner,
    pub role: TransferRole,
    pub transfer_direction: TransferDirection,
    pub connector_instance: TransferContextConnectorRole,
    /// The address the message carries (a push sink, or the pull endpoint on start).
    pub data_address: Option<DataAddress>,
    /// The address this side handed to the peer, once a start resolved it.
    pub resolved_data_address: Option<DataAddress>,
    pub is_restart: bool,
    /// Diagnostic/lifecycle reason describing why this control-plane transition ran.
    pub reason: Option<String>,
}

impl DataplaneHookContext {
    pub fn owner(&self) -> Owner {
        self.owner.clone()
    }

    /// The message's data address as the dataplane takes it; the endpoint is required there.
    pub fn dataplane_address(&self) -> Option<DataplaneAddress> {
        self.data_address
            .clone()
            .map(|address| DataplaneAddress::from(DataAddressDto::from(address)))
    }

    /// Continuation carrying this hook's diagnostic reason.
    pub fn continuation(&self) -> DataplaneContinuation {
        DataplaneContinuation::with_reason(self.process_id.clone(), self.reason.clone())
    }

    /// Continuation for an intermediate stop immediately before completion or termination.
    pub fn stopping_continuation(&self) -> DataplaneContinuation {
        let reason = self
            .reason
            .as_ref()
            .map(|r| format!("Stopping before final transition; {r}"));
        DataplaneContinuation::with_reason(self.process_id.clone(), reason)
    }

    /// Formats a human-readable dataplane log reason from DSP/RPC message fields.
    pub fn format_message_reason(
        message: &TransferDSPMessageType,
        origin: &str,
        code: Option<&str>,
        reasons: &[String],
        is_restart: bool,
    ) -> Option<String> {
        let action = match message {
            TransferDSPMessageType::TransferStartMessage if is_restart => {
                format!("Transfer resumed ({origin}: TransferStartMessage)")
            }
            TransferDSPMessageType::TransferStartMessage => {
                format!("Transfer started ({origin}: TransferStartMessage)")
            }
            TransferDSPMessageType::TransferSuspensionMessage => {
                format!("Transfer suspended ({origin}: TransferSuspensionMessage)")
            }
            TransferDSPMessageType::TransferCompletionMessage => {
                format!("Transfer completed ({origin}: TransferCompletionMessage)")
            }
            TransferDSPMessageType::TransferTerminationMessage => {
                format!("Transfer terminated ({origin}: TransferTerminationMessage)")
            }
            TransferDSPMessageType::TransferRequestMessage => {
                format!("Transfer requested ({origin}: TransferRequestMessage)")
            }
            _ => return None,
        };

        let clean_reasons: Vec<&str> = reasons
            .iter()
            .map(|r| r.trim())
            .filter(|r| !r.is_empty())
            .collect();
        let code = code.map(str::trim).filter(|c| !c.is_empty());

        match (code, clean_reasons.is_empty()) {
            (Some(c), false) => Some(format!("{action} [code={c}]: {}", clean_reasons.join("; "))),
            (Some(c), true) => Some(format!("{action} [code={c}]")),
            (None, false) => Some(format!("{action}: {}", clean_reasons.join("; "))),
            (None, true) => Some(action),
        }
    }
}

impl From<DataAddressDto> for DataplaneAddress {
    fn from(dto: DataAddressDto) -> Self {
        Self::from(&dto)
    }
}

impl From<&DataAddressDto> for DataplaneAddress {
    fn from(dto: &DataAddressDto) -> Self {
        DataplaneAddress {
            endpoint_type: dto.endpoint_type.clone(),
            endpoint: dto.endpoint.clone().unwrap_or_default(),
            authorization_type: dto
                .endpoint_properties
                .as_deref()
                .and_then(|ps| ps.iter().find(|p| p.name == "authType"))
                .map(|p| p.value.clone()),
            authorization: dto
                .endpoint_properties
                .as_deref()
                .and_then(|ps| ps.iter().find(|p| p.name == "authorization"))
                .map(|p| p.value.clone()),
        }
    }
}

impl From<DataplaneAddress> for DataAddressDto {
    fn from(addr: DataplaneAddress) -> Self {
        let mut props: Vec<EndpointPropertyDto> = Vec::new();
        if let Some(auth_type) = addr.authorization_type {
            props.push(EndpointPropertyDto {
                name: "authType".to_string(),
                value: auth_type,
            });
        }
        if let Some(authorization) = addr.authorization {
            props.push(EndpointPropertyDto {
                name: "authorization".to_string(),
                value: authorization,
            });
        }
        DataAddressDto {
            endpoint_type: addr.endpoint_type,
            endpoint: Some(addr.endpoint),
            endpoint_properties: if props.is_empty() { None } else { Some(props) },
        }
    }
}

/// Dataplane side of each DSP transfer message, split into a `_pre` and a `_post` hook.
/// A hook may return the data address to send to the peer.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DataPlaneFacadeTrait: Send + Sync {
    async fn on_transfer_request_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_request_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_start_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_start_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_suspension_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_suspension_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_completion_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_completion_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_termination_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_termination_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>>;
}
