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

use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::entities::dsp_context::TransferDSPContextDomain;
use crate::protocols::dsp::facades::dataplane_facade::{
    consumer_pull::ConsumerPullStrategy, consumer_push::ConsumerPushStrategy,
    provider_pull::ProviderPullStrategy, provider_push::ProviderPushStrategy,
};
use common::dsp_common::data_address::DataAddress;
use dataplane::DataplaneManager;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub(super) trait DataPlaneStrategy: Send + Sync {
    async fn on_request_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_request_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_start_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_start_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<Option<DataAddressDto>>;

    async fn on_suspend_pre(&self, ctx: &TransferDSPContextDomain, mgr: &DataplaneManager)
    -> Outcome<()>;

    async fn on_suspend_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_complete_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_complete_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_terminate_pre(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;

    async fn on_terminate_post(
        &self,
        ctx: &TransferDSPContextDomain,
        mgr: &DataplaneManager,
    ) -> Outcome<()>;
}

static CONSUMER_PULL: ConsumerPullStrategy = ConsumerPullStrategy;
static CONSUMER_PUSH: ConsumerPushStrategy = ConsumerPushStrategy;
static PROVIDER_PULL: ProviderPullStrategy = ProviderPullStrategy;
static PROVIDER_PUSH: ProviderPushStrategy = ProviderPushStrategy;

// Capture and factory of strategy once TransferProcess exists
pub(super) fn strategy_for(
    role: TransferRole,
    direction: TransferDirection,
) -> &'static dyn DataPlaneStrategy {
    match (role, direction) {
        (TransferRole::Consumer, TransferDirection::Pull) => &CONSUMER_PULL,
        (TransferRole::Consumer, TransferDirection::Push) => &CONSUMER_PUSH,
        (TransferRole::Provider, TransferDirection::Pull) => &PROVIDER_PULL,
        // Provider/Push, and Relay (not a data-plane role) fall back to provider push.
        _ => &PROVIDER_PUSH,
    }
}

// Capture and factory of strategy when it's being created for first time in provider
// Based on DSP, if PUSH DataAddress should be in message, otherwise is PULL
pub(super) fn strategy_for_request_pre(
    data_address: &Option<DataAddress>,
) -> &'static dyn DataPlaneStrategy {
    if data_address.is_some() {
        &CONSUMER_PUSH
    } else {
        &CONSUMER_PULL
    }
}
