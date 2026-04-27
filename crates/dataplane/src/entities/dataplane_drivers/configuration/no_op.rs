use crate::entities::dataplane_drivers::{DriverAuthenticatorTrait, DriverProxyConfiguratorTrait};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct NoOpProxyConfigurator;

#[async_trait::async_trait]
impl DriverProxyConfiguratorTrait for NoOpProxyConfigurator {
    async fn configure_proxy(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context.clone())
    }
}
