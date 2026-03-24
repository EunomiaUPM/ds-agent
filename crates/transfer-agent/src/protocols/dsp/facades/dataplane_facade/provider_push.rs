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

use crate::protocols::dsp::facades::dataplane_facade::strategy::{
    execute_command, ingress_as_data_address, to_dataplane_address, DataPlaneStrategy,
};
use crate::protocols::dsp::facades::dataplane_facade::DataAddressDto;
use connector::ConnectorInstanceDto;
use dataplane::{DataplaneCommand, DataplaneInitCommandType, DataplaneManager};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub(super) struct ProviderPushStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ProviderPushStrategy {
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
        connector_instance: &Option<ConnectorInstanceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<()> {
        // Init provider DP with the connector, then immediately set egress to the
        // consumer's ingest URL (supplied in the TransferRequest DataAddress field).
        // The autonomous chain (Init→Ready) runs inside SetInit, so the process is
        // Ready before SetEgress fires.
        let connector_instance = connector_instance
            .as_ref()
            .ok_or(Errors::crazy("Connector instance should be defined", None))?;
        execute_command(
            mgr,
            transfer_id,
            DataplaneCommand::SetInit(DataplaneInitCommandType::Provider {
                connector_instance: connector_instance.id.clone(),
                data_address: None,
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
        Ok(())
    }

    async fn on_start_pre(
        &self,
        mgr: &DataplaneManager,
        proxy_base: &str,
        transfer_id: &Urn,
    ) -> Outcome<Option<DataAddressDto>> {
        // Activate the DP. The return value is unused in PUSH mode but kept
        // consistent with the provider_pull counterpart.
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
