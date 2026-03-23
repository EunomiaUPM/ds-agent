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
    DataPlaneStrategy, execute_command, ingress_as_data_address, to_dataplane_address,
};
use connector::ConnectorInstanceDto;
use dataplane::{DataplaneCommand, DataplaneInitCommandType, DataplaneManager};
use urn::Urn;
use ymir::errors::Outcome;

pub(super) struct ConsumerPullStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ConsumerPullStrategy {
    async fn on_request_pre(
        &self,
        mgr: &DataplaneManager,
        _proxy_base: &str,
        transfer_id: &Urn,
        _data_address: &Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        // Init consumer DP without a data address; the provider will send the proxy URL
        // later in the TransferStart message.
        execute_command(
            mgr,
            transfer_id,
            DataplaneCommand::SetInit(DataplaneInitCommandType::Consumer { data_address: None }),
        )
        .await?;
        Ok(None)
    }

    async fn on_request_post(
        &self,
        _mgr: &DataplaneManager,
        _proxy_base: &str,
        _transfer_id: &Urn,
        _connector_instance: &ConnectorInstanceDto,
        _data_address: &Option<DataAddressDto>,
    ) -> Outcome<()> {
        Ok(()) // not called for consumer
    }

    async fn on_start_pre(
        &self,
        _mgr: &DataplaneManager,
        _proxy_base: &str,
        _transfer_id: &Urn,
    ) -> Outcome<Option<DataAddressDto>> {
        Ok(None) // not called for consumer
    }

    async fn on_start_post(
        &self,
        mgr: &DataplaneManager,
        proxy_base: &str,
        transfer_id: &Urn,
        data_address: Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        // Set egress to the provider's proxy URL received in TransferStartMessage,
        // then activate the DP session.
        if let Some(ref da) = data_address {
            if da.endpoint.is_some() {
                execute_command(
                    mgr,
                    transfer_id,
                    DataplaneCommand::SetEgress { data_address: to_dataplane_address(da) },
                )
                .await?;
            }
        }
        execute_command(mgr, transfer_id, DataplaneCommand::SetStarted).await?;
        ingress_as_data_address(mgr, proxy_base, transfer_id).await
    }
}
