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

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::facades::dataplane_facade::strategy::DataPlaneStrategy;
use crate::protocols::dsp::facades::dataplane_facade::DataAddressDto;
use dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneManager,
};
use std::str::FromStr;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub(super) struct ProviderPushStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ProviderPushStrategy {
    async fn on_request_pre(
        &self,
        _ctx: &DspTransferContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // noop
        Ok(None)
    }

    async fn on_request_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let id = process_urn(ctx, "provider push request_post")?;
        let connector_instance = ctx
            .connector_instance
            .as_ref()
            .ok_or_else(|| Errors::crazy("Connector instance should be defined", None))?;
        let data_address_dto = ctx
            .input_data_address
            .as_ref()
            .ok_or_else(|| Errors::crazy("Data address instance should be defined", None))?;
        let data_address: DataplaneAddress = data_address_dto.into();
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
        // _res comes with dataaddress also, but not used in implementation
        Ok(())
    }

    async fn on_start_pre(
        &self,
        _ctx: &DspTransferContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        Ok(None)
    }

    async fn on_start_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // in this case restart works same
        let id = process_urn(ctx, "provider push start_pre")?;
        mgr.execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
            transfer_dto_urn: id,
        }))
        .await?;
        Ok(None)
    }

    async fn on_suspend_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
        Ok(())
    }

    async fn on_suspend_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let id = process_urn(ctx, "provider push suspend_post")?;
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: id,
        }))
        .await?;
        Ok(())
    }

    async fn on_complete_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn on_complete_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "provider push complete_post")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_terminate_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn on_terminate_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "provider push terminate_post")?,
        }))
        .await?;
        Ok(())
    }
}

fn process_urn(ctx: &DspTransferContext, location: &str) -> Outcome<Urn> {
    let id = &ctx
        .process
        .as_ref()
        .ok_or_else(|| Errors::crazy(format!("process required for {location}"), None))?
        .inner
        .id;
    Ok(Urn::from_str(id)?)
}
