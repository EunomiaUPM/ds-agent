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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::facades::dataplane_facade::{
    consumer_pull::ConsumerPullStrategy, consumer_push::ConsumerPushStrategy,
    provider_pull::ProviderPullStrategy, provider_push::ProviderPushStrategy, DataAddressDto,
};
use dataplane::DataplaneManager;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub(super) trait DataPlaneStrategy: Send + Sync {
    async fn on_request_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_request_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_start_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_start_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_suspend_pre(&self, ctx: &DspTransferContext, mgr: &DataplaneManager)
        -> Outcome<()>;

    async fn on_suspend_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_complete_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_complete_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_terminate_pre(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_terminate_post(
        &self,
        ctx: &DspTransferContext,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;
}

static CONSUMER_PULL: ConsumerPullStrategy = ConsumerPullStrategy;
static CONSUMER_PUSH: ConsumerPushStrategy = ConsumerPushStrategy;
static PROVIDER_PULL: ProviderPullStrategy = ProviderPullStrategy;
static PROVIDER_PUSH: ProviderPushStrategy = ProviderPushStrategy;

pub(super) fn strategy_for(process: &TransferProcessDto) -> &'static dyn DataPlaneStrategy {
    match (
        process.inner.role.as_str(),
        process.inner.transfer_direction.as_str(),
    ) {
        ("Consumer", "Pull") => &CONSUMER_PULL,
        ("Consumer", _) => &CONSUMER_PUSH,
        ("Provider", "Pull") => &PROVIDER_PULL,
        _ => &PROVIDER_PUSH,
    }
}

pub(super) fn strategy_for_request_pre(
    data_address: &Option<DataAddressDto>,
) -> &'static dyn DataPlaneStrategy {
    if data_address.is_some() {
        &CONSUMER_PUSH
    } else {
        &CONSUMER_PULL
    }
}
