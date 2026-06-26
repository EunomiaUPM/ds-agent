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

use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use connector::AuthenticationConfig;
use crate::errors::DataplaneError;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct BearerTokenAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for BearerTokenAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or_else(|| DataplaneError::ConnectorNotAvailable)?;

        let AuthenticationConfig::BearerToken { token } = connector.authentication_config.clone()
        else {
            return Err(DataplaneError::AuthConfigMismatch {
                expected: "BearerToken".to_string(),
            }
            .into());
        };

        let resolved = token.resolve().await?;
        dbg!(&resolved);
        dbg!(&token);


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
    use crate::entities::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
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
        let ctx = no_auth_context().await;
        let result = BearerTokenAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_error_without_connector() {
        let ctx = consumer_context().await;
        let result = BearerTokenAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }
}
