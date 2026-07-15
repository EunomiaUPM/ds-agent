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

use crate::protocols::dsp::entities::context_dsp::TransferDSPContextDomain;

mod consumer_pull;
mod consumer_push;
pub(crate) mod dataplane_facade;
mod provider_pull;
mod provider_push;
mod strategy;

use crate::protocols::dsp::entities::data_address::{DataAddressDto, EndpointPropertyDto};
use common::dsp_common::data_address::DataAddress;
use dataplane::DataplaneAddress;
use ymir::errors::Outcome;

/// The message's `DataAddress` (common wire type) → data-plane address. A free
/// function rather than a `From` impl: both types are foreign, so the orphan rule
/// forbids the trait impl. Endpoint properties are flattened by name
/// (`authType`, `authorization`), same as the DTO path.
pub(super) fn to_dataplane_address(addr: &DataAddress) -> DataplaneAddress {
    let prop = |name: &str| {
        addr.endpoint_properties
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.value.clone())
    };
    DataplaneAddress {
        endpoint_type: addr.endpoint_type.clone(),
        endpoint: addr.endpoint.clone(),
        authorization_type: prop("authType"),
        authorization: prop("authorization"),
    }
}

impl From<DataAddressDto> for DataplaneAddress {
    fn from(dto: DataAddressDto) -> Self {
        DataplaneAddress {
            endpoint_type: dto.endpoint_type,
            endpoint: dto.endpoint.unwrap_or_default(),
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

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataPlaneFacadeTrait: Send + Sync {
    // TransferRequest ───
    async fn on_transfer_request_pre(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_request_post(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    // TransferStart ───

    async fn on_transfer_start_pre(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_start_post(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    // TransferSuspension ───

    async fn on_transfer_suspension_pre(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_suspension_post(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    // TransferCompletion ───

    async fn on_transfer_completion_pre(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_completion_post(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    // TransferTermination ───

    async fn on_transfer_termination_pre(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_termination_post(
        &self,
        ctx: &TransferDSPContextDomain,
    ) -> Outcome<Option<DataAddressDto>>;
}
