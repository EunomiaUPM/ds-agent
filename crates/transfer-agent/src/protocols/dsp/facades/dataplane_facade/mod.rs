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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use connector::ConnectorInstanceDto;
use dataplane::DataplaneAddress;
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataPlaneFacadeTrait: Send + Sync {
    // ─── TransferRequest ───
    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_request_post(
        &self,
        transfer_process: &TransferProcessDto,
        connector_instance: &Option<ConnectorInstanceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<()>;

    // ─── TransferStart ───

    async fn on_transfer_start_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_transfer_start_post(
        &self,
        transfer_process: &TransferProcessDto,
        data_address: Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>>;

    // ─── TransferSuspension ───

    async fn on_transfer_suspension_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()>;

    async fn on_transfer_suspension_post(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()>;

    // ─── TransferCompletion ───

    async fn on_transfer_completion_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()>;

    async fn on_transfer_completion_post(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()>;

    // ─── TransferTermination ───

    async fn on_transfer_termination_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()>;

    async fn on_transfer_termination_post(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()>;

    // ─── Config updates ───

    /// Update the egress config for a transfer (e.g. after receiving peer's DataAddress)
    async fn set_egress(
        &self,
        transfer_process: &TransferProcessDto,
        data_address: DataplaneAddress,
    ) -> Outcome<()>;
}
