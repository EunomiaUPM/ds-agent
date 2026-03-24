/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
use crate::protocols::dsp::facades::dataplane_facade::strategy::{
    execute_command, strategy_for, strategy_for_request_pre,
};
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use connector::ConnectorInstanceDto;
use dataplane::{DataplaneAddress, DataplaneCommand, DataplaneManager};
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct DspDataPlaneFacade {
    dataplane_manager: Arc<DataplaneManager>,
    proxy_base_url: String,
}

impl DspDataPlaneFacade {
    pub fn new(
        dataplane_manager: Arc<DataplaneManager>,
        proxy_base_url: String,
    ) -> DspDataPlaneFacade {
        DspDataPlaneFacade { dataplane_manager, proxy_base_url }
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DspDataPlaneFacade {
    // ─── TransferRequest ───

    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        strategy_for_request_pre(data_address)
            .on_request_pre(&self.dataplane_manager, &self.proxy_base_url, transfer_id, data_address)
            .await
    }

    async fn on_transfer_request_post(
        &self,
        transfer_process: &TransferProcessDto,
        connector_instance: &Option<ConnectorInstanceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<()> {
        let id = Urn::from_str(&transfer_process.inner.id)?;
        strategy_for(transfer_process)
            .on_request_post(
                &self.dataplane_manager,
                &self.proxy_base_url,
                &id,
                connector_instance,
                data_address,
            )
            .await
    }

    // ─── TransferStart ───

    async fn on_transfer_start_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<Option<DataAddressDto>> {
        let id = Urn::from_str(&transfer_process.inner.id)?;
        strategy_for(transfer_process)
            .on_start_pre(&self.dataplane_manager, &self.proxy_base_url, &id)
            .await
    }

    async fn on_transfer_start_post(
        &self,
        transfer_process: &TransferProcessDto,
        data_address: Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        let id = Urn::from_str(&transfer_process.inner.id)?;
        strategy_for(transfer_process)
            .on_start_post(&self.dataplane_manager, &self.proxy_base_url, &id, data_address)
            .await
    }

    // ─── TransferSuspension → SetStopped ───

    async fn on_transfer_suspension_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetStopped,
        )
        .await
    }

    async fn on_transfer_suspension_post(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetStopped,
        )
        .await
    }

    // ─── TransferCompletion → SetTerminated ───

    async fn on_transfer_completion_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetTerminated,
        )
        .await
    }

    async fn on_transfer_completion_post(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetTerminated,
        )
        .await
    }

    // ─── TransferTermination → SetTerminated ───

    async fn on_transfer_termination_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetTerminated,
        )
        .await
    }

    async fn on_transfer_termination_post(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetTerminated,
        )
        .await
    }

    // ─── Config updates ───

    async fn set_egress(
        &self,
        transfer_process: &TransferProcessDto,
        data_address: DataplaneAddress,
    ) -> Outcome<()> {
        execute_command(
            &self.dataplane_manager,
            &Urn::from_str(&transfer_process.inner.id)?,
            DataplaneCommand::SetEgress { data_address },
        )
        .await
    }
}
