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

use crate::protocols::dsp::entities::context_dsp::TransferDSPContextDomain as DspTransferContext;

use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::facades::dataplane_facade::strategy::DataPlaneStrategy;
use crate::protocols::dsp::facades::dataplane_facade::to_dataplane_address;
use dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneManager,
};
use ymir::errors::{Errors, Outcome};

pub(super) struct ConsumerPullStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ConsumerPullStrategy {
    async fn on_request_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        let transfer_id = ctx.process_urn("consumer pull request_pre")?;
        let cmd = DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsConsumer {
            transfer_process_id: transfer_id.clone(),
            direction: DataplaneInitCommandDirection::Pull { data_address: None },
        });
        mgr.execute_command(cmd).await?;
        Ok(None)
    }

    async fn on_request_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
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
        let id = ctx.process_urn("consumer pull start_post")?;
        let continuation = DataplaneContinuation {
            transfer_dto_urn: id,
        };
        if !ctx.is_restart {
            let dataplane: DataplaneAddress = ctx
                .typed
                .fields
                .data_address
                .clone()
                .map(|addr| to_dataplane_address(&addr))
                .ok_or_else(|| {
                    Errors::crazy(
                        "Dataplane_address required for consumer pull start post",
                        None,
                    )
                })?;
            let cmd = DataplaneCommand::SetConfiguring((continuation, dataplane));
            mgr.execute_command(cmd).await?;
            return Ok(None);
        }
        if ctx.is_restart {
            let cmd = DataplaneCommand::SetStarted(continuation);
            mgr.execute_command(cmd).await?;
            return Ok(None);
        }
        Ok(None)
    }

    async fn on_suspend_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn on_suspend_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: ctx.process_urn("consumer pull suspend_post")?,
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
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: ctx.process_urn("consumer pull complete_post")?,
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
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: ctx.process_urn("consumer pull terminate_post")?,
        }))
        .await?;
        Ok(())
    }
}
