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

use crate::protocols::dsp::facades::dataplane_facade::strategy::{
    execute_command, ingress_as_data_address, to_dataplane_address, DataPlaneStrategy,
};
use crate::protocols::dsp::facades::dataplane_facade::DataAddressDto;
use connector::ConnectorInstanceDto;
use dataplane::{DataplaneCommand, DataplaneInitCommandType, DataplaneManager};
use urn::Urn;
use ymir::errors::Outcome;

pub(super) struct ConsumerPushStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ConsumerPushStrategy {
    async fn on_request_pre(
        &self,
        mgr: &DataplaneManager,
        proxy_base: &str,
        transfer_id: &Urn,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        // Init consumer DP signalling PUSH mode; set egress to the data client's
        // original destination; return the auto-generated ingest URL to replace the
        // outgoing DataAddress field so the provider knows where to push data.
        let init_da = data_address.as_ref().map(to_dataplane_address);
        execute_command(
            mgr,
            transfer_id,
            DataplaneCommand::SetInit(DataplaneInitCommandType::Consumer {
                data_address: init_da,
            }),
        )
        .await?;
        if let Some(da) = data_address {
            if da.endpoint.is_some() {
                execute_command(
                    mgr,
                    transfer_id,
                    DataplaneCommand::SetEgress {
                        data_address: to_dataplane_address(da),
                    },
                )
                .await?;
            }
        }
        ingress_as_data_address(mgr, proxy_base, transfer_id).await
    }

    async fn on_request_post(
        &self,
        _mgr: &DataplaneManager,
        _proxy_base: &str,
        _transfer_id: &Urn,
        _connector_instance: &Option<ConnectorInstanceDto>,
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
        _data_address: Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>> {
        // Egress was already set to the client destination in on_request_pre; just activate.
        execute_command(mgr, transfer_id, DataplaneCommand::SetStarted).await?;
        ingress_as_data_address(mgr, proxy_base, transfer_id).await
    }
}
