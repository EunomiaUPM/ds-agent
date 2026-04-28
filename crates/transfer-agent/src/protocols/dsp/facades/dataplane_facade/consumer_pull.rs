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

pub(super) struct ConsumerPullStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ConsumerPullStrategy {
    async fn on_request_pre(
        &self,
        _ctx: &DspTransferContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        Ok(None)
    }

    async fn on_request_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let transfer_id = ctx.local_process_id.as_ref().ok_or_else(|| {
            Errors::crazy(
                "local_process_id required for consumer pull request_post",
                None,
            )
        })?;
        let cmd = DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsConsumer {
            transfer_process_id: transfer_id.clone(),
            direction: DataplaneInitCommandDirection::Pull {
                data_address: DataplaneAddress {
                    // TODO - Optional...
                    endpoint_type: "".to_string(),
                    endpoint: "".to_string(),
                    authorization_type: None,
                    authorization: None,
                },
            },
        });
        mgr.execute_command(cmd).await?;
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
        let id = process_urn(ctx, "consumer pull start_post")?;
        let cmd = DataplaneCommand::SetStarted(
            DataplaneContinuation {
                transfer_dto_urn: id,
            },
            ctx.input_data_address.clone().map(|addr| addr.into()), // set egress
        );
        mgr.execute_command(cmd).await?;
        Ok(None)
    }

    async fn on_suspend_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "consumer pull suspend_pre")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_suspend_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "consumer pull suspend_post")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_complete_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "consumer pull complete_pre")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_complete_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "consumer pull complete_post")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_terminate_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "consumer pull terminate_pre")?,
        }))
        .await?;
        Ok(())
    }

    async fn on_terminate_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
            transfer_dto_urn: process_urn(ctx, "consumer pull terminate_post")?,
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
