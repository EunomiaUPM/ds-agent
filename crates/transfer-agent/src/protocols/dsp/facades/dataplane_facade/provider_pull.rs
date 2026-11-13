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
use crate::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;

use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::facades::dataplane_facade::strategy::DataPlaneStrategy;
use dataplane::{
    DataplaneCommand, DataplaneCommandResponse, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneManager,
};
use ymir::errors::{Errors, Outcome};

pub(super) struct ProviderPullStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ProviderPullStrategy {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_request_pre(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // noop
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_request_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let id = ctx.process_id.clone();
        let TransferContextConnectorRole::ProviderHavingConnector(connector_instance) =
            &ctx.connector_instance
        else {
            return Err(Errors::crazy("Connector instance should be defined", None));
        };
        let cmd = DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsProvider {
            transfer_process_id: id,
            owner: ctx.owner(),
            connector_instance: connector_instance.as_ref().clone(),
            direction: DataplaneInitCommandDirection::Pull { data_address: None },
        });
        let _res = mgr.execute_command(cmd).await?;
        // _res comes with dataaddress also, but not used in implementation
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_start_pre(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // First start and restart alike hand the consumer the process's pull endpoint.
        let res = mgr
            .execute_command(DataplaneCommand::GetAssociated(ctx.continuation()))
            .await?;
        if let DataplaneCommandResponse::OkWithAddress(address) = res {
            Ok(Some(address.into()))
        } else {
            Err(Errors::crazy("Data address instance should be defined", None))
        }
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_start_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        if !ctx.is_restart && ctx.resolved_data_address.is_none() {
            return Err(Errors::crazy(
                "Dataplane_address required for provider pull start_post",
                None,
            ));
        }
        mgr.execute_command(DataplaneCommand::SetStarted(ctx.continuation()))
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_suspend_pre(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<()> {
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_suspend_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(ctx.continuation()))
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_complete_pre(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<()> {
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_complete_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(ctx.stopping_continuation()))
            .await?;
        mgr.execute_command(DataplaneCommand::SetTerminating(ctx.continuation()))
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_terminate_pre(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<()> {
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_terminate_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        mgr.execute_command(DataplaneCommand::SetStopped(ctx.stopping_continuation()))
            .await?;
        mgr.execute_command(DataplaneCommand::SetTerminating(ctx.continuation()))
            .await?;
        Ok(())
    }
}
