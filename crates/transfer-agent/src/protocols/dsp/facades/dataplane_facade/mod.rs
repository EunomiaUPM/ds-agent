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
pub(crate) mod dataplane_facade;
mod provider_pull;
mod provider_push;
mod strategy;

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use dataplane::DataplaneAddress;
use ymir::errors::Outcome;

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

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataPlaneFacadeTrait: Send + Sync {
    // ─── TransferRequest ───
    async fn on_transfer_request_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_request_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    // ─── TransferStart ───

    async fn on_transfer_start_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_start_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    // ─── TransferSuspension ───

    async fn on_transfer_suspension_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_suspension_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    // ─── TransferCompletion ───

    async fn on_transfer_completion_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_completion_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    // ─── TransferTermination ───

    async fn on_transfer_termination_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_termination_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>>;
}
