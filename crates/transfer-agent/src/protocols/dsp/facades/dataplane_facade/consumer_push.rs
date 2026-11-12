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

use crate::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;
use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::facades::dataplane_facade::strategy::DataPlaneStrategy;
use dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneCommandResponse, DataplaneContinuation,
    DataplaneInitCommandDirection, DataplaneInitCommandTypes, DataplaneManager,
};
use ymir::errors::{Errors, Outcome};

pub(super) struct ConsumerPushStrategy;

#[async_trait::async_trait]
impl DataPlaneStrategy for ConsumerPushStrategy {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_request_pre(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        let transfer_id = ctx.process_id.clone();
        let data_address: DataplaneAddress = ctx
            .dataplane_address()
            .ok_or_else(|| Errors::crazy("Data address instance should be defined", None))?;
        let res = mgr
            .execute_command(DataplaneCommand::SetInit(
                DataplaneInitCommandTypes::AsConsumer {
                    transfer_process_id: transfer_id.clone(),
                    owner: ctx.owner(),
                    direction: DataplaneInitCommandDirection::Push {
                        data_address: Some(data_address),
                    },
                },
            ))
            .await?;

        if let DataplaneCommandResponse::OkWithAddress(address) = res {
            Ok(Some(address.into()))
        } else {
            Err(Errors::crazy(
                "Dataplane should retrieve Dataplane ad",
                None,
            ))
        }
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_request_post(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_start_pre(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        // noop
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_start_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>> {
        let id = ctx.process_id.clone();
        mgr.execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
            transfer_dto_urn: id,
        }))
        .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_suspend_pre(
        &self,
        _ctx: &DataplaneHookContext,
        _mgr: &DataplaneManager,
    ) -> Outcome<()> {
        // noop
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_suspend_post(
        &self,
        ctx: &DataplaneHookContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()> {
        let id = ctx.process_id.clone();
        mgr.execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: id,
        }))
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
        let cont = DataplaneContinuation {
            transfer_dto_urn: ctx.process_id.clone(),
        };
        mgr.execute_command(DataplaneCommand::SetTerminating(cont))
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
        let cont = DataplaneContinuation {
            transfer_dto_urn: ctx.process_id.clone(),
        };
        mgr.execute_command(DataplaneCommand::SetTerminating(cont))
            .await?;
        Ok(())
    }
}
