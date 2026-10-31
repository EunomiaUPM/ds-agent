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
use common::dsp_common::data_address::DataAddress;
use ymir::oauth::Owner;
use dataplane::DataplaneAddress;
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
