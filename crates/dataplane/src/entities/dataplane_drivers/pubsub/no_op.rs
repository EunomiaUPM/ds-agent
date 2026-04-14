use crate::entities::dataplane_drivers::{
    DriverAuthenticatorTrait, DriverProxyConfiguratorTrait, DriverPubSubTrait,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct NoOpPubSubscriber;

#[async_trait::async_trait]
impl DriverPubSubTrait for NoOpPubSubscriber {
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context.clone())
    }

    async fn unsubscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        dbg!("{:}", &context);
        Ok(context.clone())
    }
}
