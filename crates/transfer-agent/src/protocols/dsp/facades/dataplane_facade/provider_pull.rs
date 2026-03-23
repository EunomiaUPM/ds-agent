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

use crate::protocols::dsp::facades::dataplane_facade::DataAddressDto;
use crate::protocols::dsp::facades::dataplane_facade::strategy::{
    DataPlaneStrategy, execute_command, ingress_as_data_address,
};
use connector::ConnectorInstanceDto;
use dataplane::{DataplaneCommand, DataplaneInitCommandType, DataplaneManager};
use urn::Urn;
use ymir::errors::Outcome;

pub(super) struct ProviderPullStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ProviderPullStrategy {
    async fn on_request_pre(
        &self,
        _mgr: &DataplaneManager,
        _proxy_base: &str,
        _transfer_id: &Urn,
        _data_address: &Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        Ok(None) // not called for provider
    }

    async fn on_request_post(
        &self,
        mgr: &DataplaneManager,
        _proxy_base: &str,
        transfer_id: &Urn,
        connector_instance: &ConnectorInstanceDto,
        _data_address: &Option<DataAddressDto>,
    ) -> Outcome<()> {
        // Init provider DP with the connector. No egress yet — the consumer will
        // send their ingest URL inside the TransferStart ack (consumer on_start_post).
        execute_command(
            mgr,
            transfer_id,
            DataplaneCommand::SetInit(DataplaneInitCommandType::Provider {
                connector_instance: connector_instance.id.clone(),
                data_address: None,
            }),
        )
        .await
    }

    async fn on_start_pre(
        &self,
        mgr: &DataplaneManager,
        proxy_base: &str,
        transfer_id: &Urn,
    ) -> Outcome<Option<DataAddressDto>> {
        // Activate the DP and return the proxy listener URL to embed in
        // TransferStartMessage so the consumer knows where to fetch data from.
        execute_command(mgr, transfer_id, DataplaneCommand::SetStarted).await?;
        ingress_as_data_address(mgr, proxy_base, transfer_id).await
    }

    async fn on_start_post(
        &self,
        _mgr: &DataplaneManager,
        _proxy_base: &str,
        _transfer_id: &Urn,
        _data_address: Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        Ok(None) // not called for provider
    }
}
