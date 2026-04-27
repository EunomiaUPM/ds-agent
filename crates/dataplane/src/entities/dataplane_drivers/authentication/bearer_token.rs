use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use connector::AuthenticationConfig;
use ymir::errors::{Errors, Outcome};

#[derive(Debug)]
pub struct BearerTokenAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for BearerTokenAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("Connector not available", None))?;

        let AuthenticationConfig::BearerToken { token } =
            connector.authentication_config.clone()
        else {
            return Err(Errors::crazy(
                "Connector auth config should be type BEARER_TOKEN",
                None,
            ));
        };

        let resolved = token.resolve().await?;
        let mut ctx = context.clone();
        ctx.set_runtime(DataplaneRuntime {
            auth: ResolvedAuthCredentials::BearerToken { token: resolved },
            ..Default::default()
        });
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::dataplane_manager_ref::dataplane_runtime::ResolvedAuthCredentials;
    use crate::test_fixtures::{bearer_context, consumer_context, no_auth_context};

    #[tokio::test]
    async fn resolves_plain_bearer_token() {
        let ctx = bearer_context("secret-token").await;
        let result = BearerTokenAuthenticator.authenticate(&ctx).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let runtime = updated.runtime().expect("runtime must be set after auth");
        match &runtime.auth {
            ResolvedAuthCredentials::BearerToken { token } => {
                assert_eq!(token, "secret-token");
            }
            other => panic!("Expected BearerToken, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn returns_error_for_wrong_auth_type() {
        // NoAuth context -> BearerToken authenticator should reject it
        let ctx = no_auth_context().await;
        let result = BearerTokenAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_error_without_connector() {
        // Consumer pull context has no connector instance
        let ctx = consumer_context().await;
        let result = BearerTokenAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }
}
