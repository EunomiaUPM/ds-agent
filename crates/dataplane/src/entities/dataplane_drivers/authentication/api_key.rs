use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use ymir::errors::{Errors, Outcome};
use connector::AuthenticationConfig;

#[derive(Debug)]
pub struct ApiKeyAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for ApiKeyAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        if let Some(connector) = context.connector_instance() {
            if let AuthenticationConfig::ApiKey {
                key,
                value,
                location
            } = connector.authentication_config.clone() {
                Ok(context.clone())
            } else {
                Err(Errors::crazy("Connector auth config should be type API_KEY", None))
            }
        } else {
            Err(Errors::crazy("Connector not available", None))
        }
    }
}
