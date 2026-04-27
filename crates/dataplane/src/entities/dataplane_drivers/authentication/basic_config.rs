use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use connector::AuthenticationConfig;
use ymir::errors::{Errors, Outcome};

#[derive(Debug)]
pub struct BasicConfigAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for BasicConfigAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("Connector not available", None))?;

        let AuthenticationConfig::BasicAuth(basic) = connector.authentication_config.clone() else {
            return Err(Errors::crazy(
                "Connector auth config should be type BASIC_AUTH",
                None,
            ));
        };

        let password = basic.password.resolve().await?;
        let mut ctx = context.clone();
        ctx.set_runtime(DataplaneRuntime {
            auth: ResolvedAuthCredentials::BasicAuth {
                username: basic.username,
                password,
            },
            ..Default::default()
        });
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::dataplane_manager_ref::dataplane_runtime::ResolvedAuthCredentials;
    use crate::test_fixtures::{basic_auth_context, bearer_context, consumer_context};

    #[tokio::test]
    async fn resolves_plain_basic_auth() {
        let ctx = basic_auth_context("alice", "s3cr3t").await;
        let result = BasicConfigAuthenticator.authenticate(&ctx).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let runtime = updated.runtime().expect("runtime must be set");
        match &runtime.auth {
            ResolvedAuthCredentials::BasicAuth { username, password } => {
                assert_eq!(username, "alice");
                assert_eq!(password, "s3cr3t");
            }
            other => panic!("Expected BasicAuth, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn returns_error_for_wrong_auth_type() {
        let ctx = bearer_context("token").await;
        let result = BasicConfigAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_error_without_connector() {
        let ctx = consumer_context().await;
        let result = BasicConfigAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }
}
