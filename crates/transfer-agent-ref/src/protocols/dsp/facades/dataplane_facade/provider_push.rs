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

use crate::protocols::dsp::entities::context_common::TransferContextConnectorRole;
use crate::protocols::dsp::entities::context_dsp::TransferDSPContextDomain;

use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::facades::dataplane_facade::strategy::DataPlaneStrategy;
use crate::protocols::dsp::facades::dataplane_facade::to_dataplane_address;
use dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneManager,
};
use ymir::errors::{Errors, Outcome};

pub(super) struct ProviderPushStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ProviderPushStrategy {
    async fn on_request_pre(
        &self,
        _ctx: &TransferDSPContextDomain,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // noop
        Ok(None)
    }

    async fn on_request_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let id = ctx.process_urn("provider push request_post")?;
        let TransferContextConnectorRole::ProviderHavingConnector(connector_instance) =
            &ctx.connector_instance
        else {
            return Err(Errors::crazy("Connector instance should be defined", None));
        };
        let data_address_dto = ctx
            .typed
            .fields
            .data_address
            .as_ref()
            .ok_or_else(|| Errors::crazy("Data address instance should be defined", None))?;
        let data_address: DataplaneAddress = to_dataplane_address(data_address_dto);
        let _res = mgr
            .execute_command(DataplaneCommand::SetInit(
                DataplaneInitCommandTypes::AsProvider {
                    transfer_process_id: id,
                    connector_instance: connector_instance.clone(),
                    direction: DataplaneInitCommandDirection::Push {
                        data_address: Some(data_address),
                    },
                },
            ))
            .await?;
        // _res comes with dataAddress also, but not used in implementation
        Ok(())
    }

    async fn on_start_pre(
        &self,
        _ctx: &TransferDSPContextDomain,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // noop
        Ok(None)
    }

    async fn on_start_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // in this case restart works same
        let id = ctx.process_urn("provider push start_pre")?;
        mgr.execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
            transfer_dto_urn: id,
        }))
        .await?;
        Ok(None)
    }

    async fn on_suspend_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
        Ok(())
    }

    async fn on_suspend_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let id = ctx.process_urn("provider push suspend_post")?;
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: id,
        }))
        .await?;
        Ok(())
    }

    async fn on_complete_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
        Ok(())
    }

    async fn on_complete_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: ctx.process_urn("provider push complete_post")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_terminate_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
        Ok(())
    }

    async fn on_terminate_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: ctx.process_urn("provider push terminate_post")?,
        }))
        .await?;
        Ok(())
    }
}
