use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct NoAuthAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for NoAuthAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context.clone())
    }
}
