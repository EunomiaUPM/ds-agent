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

use crate::protocols::dsp::facades::dataplane_facade::strategy::DataPlaneStrategy;
use crate::protocols::dsp::facades::dataplane_facade::DataAddressDto;
use connector::ConnectorInstanceDto;
use dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneManager,
};
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
        let connector_instance = connector_instance
            .as_ref()
            .ok_or(Errors::crazy("Connector instance should be defined", None))?;
        let data_address_dto = data_address.as_ref().ok_or(Errors::crazy(
            "Data address instance should be defined",
            None,
        ))?;
        let data_address: DataplaneAddress = data_address_dto.clone().into();
        let _ = mgr
            .execute_command(DataplaneCommand::SetInit(
                DataplaneInitCommandTypes::AsProvider {
                    transfer_process_id: transfer_id.clone(),
                    connector_instance: connector_instance.clone(),
                    direction: DataplaneInitCommandDirection::Push { data_address },
                },
            ))
            .await?;
        Ok(())
    }

    async fn on_start_pre(
        &self,
        mgr: &DataplaneManager,
        proxy_base: &str,
        transfer_id: &Urn,
    ) -> Outcome<Option<DataAddressDto>> {
        let cmd = DataplaneCommand::SetStarted(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(None)
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

    async fn on_suspend_pre(&self, mgr: &DataplaneManager, transfer_id: &Urn) -> Outcome<()> {
        let cmd = DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(())
    }

    async fn on_suspend_post(&self, mgr: &DataplaneManager, transfer_id: &Urn) -> Outcome<()> {
        let cmd = DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(())
    }

    async fn on_complete_pre(&self, mgr: &DataplaneManager, transfer_id: &Urn) -> Outcome<()> {
        let cmd = DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(())
    }

    async fn on_complete_post(&self, mgr: &DataplaneManager, transfer_id: &Urn) -> Outcome<()> {
        let cmd = DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(())
    }

    async fn on_terminate_pre(&self, mgr: &DataplaneManager, transfer_id: &Urn) -> Outcome<()> {
        let cmd = DataplaneCommand::SetTerminating(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(())
    }

    async fn on_terminate_post(&self, mgr: &DataplaneManager, transfer_id: &Urn) -> Outcome<()> {
        let cmd = DataplaneCommand::SetTerminating(DataplaneContinuation {
            transfer_dto_urn: transfer_id.clone(),
        });
        let _ = mgr.execute_command(cmd).await?;
        Ok(())
    }
}
