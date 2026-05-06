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
    DataplaneAddress, DataplaneCommand, DataplaneCommandResponse, DataplaneContinuation,
    DataplaneInitCommandDirection, DataplaneInitCommandTypes, DataplaneManager,
};
use std::str::FromStr;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub(super) struct ProviderPullStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ProviderPullStrategy {
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
        let id = process_urn(ctx, "provider pull request_post")?;
        let connector_instance = ctx
            .connector_instance
            .as_ref()
            .ok_or_else(|| Errors::crazy("Connector instance should be defined", None))?;
        let cmd = DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsProvider {
            transfer_process_id: id,
            connector_instance: connector_instance.clone(),
            direction: DataplaneInitCommandDirection::Pull { data_address: None },
        });
        let _res = mgr.execute_command(cmd).await?;
        // _res comes with dataaddress also, but not used in implementation
        Ok(())
    }

    async fn on_start_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        if !ctx.is_restart {
            let id = process_urn(ctx, "provider pull start_pre")?;
            let res = mgr
                .execute_command(DataplaneCommand::GetAssociated(DataplaneContinuation {
                    transfer_dto_urn: id,
                }))
                .await?;

            return if let DataplaneCommandResponse::OkWithAddress(address) = res {
                dbg!(&address);
                Ok(Some(address.into()))
            } else {
                Err(Errors::crazy(
                    "Data address instance should be defined",
                    None,
                ))
            };
        }
        if ctx.is_restart {
            todo!()
        }
        Ok(None)
    }

    async fn on_start_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        dbg!(&ctx);

        let id = process_urn(ctx, "provider pull start_post")?;
        if !ctx.is_restart {
            let continuation = DataplaneContinuation {
                transfer_dto_urn: id,
            };
            let dataplane: DataplaneAddress = ctx
                .resolved_data_address
                .clone()
                .map(|addr| addr.into())
                .ok_or_else(|| {
                    Errors::crazy(
                        "Dataplane_address required for provider pull start_post",
                        None,
                    )
                })?;
            let cmd = DataplaneCommand::SetStarted(continuation);
            let res = mgr.execute_command(cmd).await?;
            return Ok(None);
        }
        if ctx.is_restart {
            mgr.execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
                transfer_dto_urn: id,
            }))
            .await?;
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
            transfer_dto_urn: process_urn(ctx, "provider pull suspend_post")?,
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
            transfer_dto_urn: process_urn(ctx, "provider pull complete_post")?,
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
            transfer_dto_urn: process_urn(ctx, "provider pull terminate_post")?,
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
