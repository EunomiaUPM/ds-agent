pub(super) mod authentication;
pub(super) mod configuration;
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
