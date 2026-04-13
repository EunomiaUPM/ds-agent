use crate::entities::dataplane_drivers::{DriverAuthenticatorTrait, DriverProxyConfiguratorTrait};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct HttpConsumerPullConfigurator;

#[async_trait::async_trait]
impl DriverProxyConfiguratorTrait for HttpConsumerPullConfigurator {
    async fn configure_proxy(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        dbg!("Configuring proxy baby shark");
        Ok(context.clone())
    }
}
