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
use crate::protocols::dsp::facades::dataplane_facade::strategy::{
    strategy_for, strategy_for_request_pre,
};
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use dataplane::DataplaneManager;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct DspDataPlaneFacade {
    dataplane_manager: Arc<DataplaneManager>,
    proxy_base_url: String,
}

impl DspDataPlaneFacade {
    pub fn new(
        dataplane_manager: Arc<DataplaneManager>,
        proxy_base_url: String,
    ) -> DspDataPlaneFacade {
        DspDataPlaneFacade {
            dataplane_manager,
            proxy_base_url,
        }
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DspDataPlaneFacade {
    async fn on_transfer_request_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        strategy_for_request_pre(&ctx.input_data_address)
            .on_request_pre(ctx, &self.dataplane_manager)
            .await
    }

    async fn on_transfer_request_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx
            .process
            .as_ref()
            .ok_or_else(|| Errors::crazy("process required for on_transfer_request_post", None))?;
        strategy_for(process)
            .on_request_post(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    async fn on_transfer_start_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx
            .process
            .as_ref()
            .ok_or_else(|| Errors::crazy("process required for on_transfer_start_pre", None))?;
        strategy_for(process)
            .on_start_pre(ctx, &self.dataplane_manager)
            .await
    }

    async fn on_transfer_start_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx
            .process
            .as_ref()
            .ok_or_else(|| Errors::crazy("process required for on_transfer_start_post", None))?;
        strategy_for(process)
            .on_start_post(ctx, &self.dataplane_manager)
            .await
    }

    async fn on_transfer_suspension_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx.process.as_ref().ok_or_else(|| {
            Errors::crazy("process required for on_transfer_suspension_pre", None)
        })?;
        strategy_for(process)
            .on_suspend_pre(ctx, &self.dataplane_manager)
            .await;
        Ok(None)
    }

    async fn on_transfer_suspension_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx.process.as_ref().ok_or_else(|| {
            Errors::crazy("process required for on_transfer_suspension_post", None)
        })?;
        strategy_for(process)
            .on_suspend_post(ctx, &self.dataplane_manager)
            .await;
        Ok(None)
    }

    async fn on_transfer_completion_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx.process.as_ref().ok_or_else(|| {
            Errors::crazy("process required for on_transfer_completion_pre", None)
        })?;
        strategy_for(process)
            .on_complete_pre(ctx, &self.dataplane_manager)
            .await;
        Ok(None)
    }

    async fn on_transfer_completion_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx.process.as_ref().ok_or_else(|| {
            Errors::crazy("process required for on_transfer_completion_post", None)
        })?;
        strategy_for(process)
            .on_complete_post(ctx, &self.dataplane_manager)
            .await;
        Ok(None)
    }

    async fn on_transfer_termination_pre(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx.process.as_ref().ok_or_else(|| {
            Errors::crazy("process required for on_transfer_termination_pre", None)
        })?;
        strategy_for(process)
            .on_terminate_pre(ctx, &self.dataplane_manager)
            .await;
        Ok(None)
    }

    async fn on_transfer_termination_post(
        &self,
        ctx: &DspTransferContext,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = ctx.process.as_ref().ok_or_else(|| {
            Errors::crazy("process required for on_transfer_termination_post", None)
        })?;
        strategy_for(process)
            .on_terminate_post(ctx, &self.dataplane_manager)
            .await;
        Ok(None)
    }
}
