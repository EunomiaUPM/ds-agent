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

pub(super) mod authentication;
pub(super) mod configuration;
pub(crate) mod proxy;
pub(super) mod pubsub;

use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use ymir::errors::Outcome;

#[derive(Clone, Debug)]
pub struct DataplaneDriver {
    pub authenticator: Arc<dyn DriverAuthenticatorTrait>,
    pub proxy_configurator: Arc<dyn DriverProxyConfiguratorTrait>,
    pub subscriber: Option<Arc<dyn DriverPubSubTrait>>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DriverAuthenticatorTrait: Send + Sync + Debug {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DriverProxyConfiguratorTrait: Send + Sync + Debug {
    async fn configure_proxy(&self, context: &DataplaneContext) -> Outcome<DataplaneContext>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DriverPubSubTrait: Send + Sync + Debug {
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext>;
    async fn unsubscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext>;
}
