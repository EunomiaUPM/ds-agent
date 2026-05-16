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
use ymir::errors::{Errors, Outcome};

#[derive(Debug)]
pub struct ApiKeyAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for ApiKeyAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("Connector not available", None))?;

        let AuthenticationConfig::ApiKey {
            key,
            value,
            location,
        } = connector.authentication_config.clone()
        else {
            return Err(Errors::crazy(
                "Connector auth config should be type API_KEY",
                None,
            ));
        };

        let resolved_value = value.resolve().await?;
        let mut ctx = context.clone();
        ctx.set_runtime(DataplaneRuntime {
            auth: ResolvedAuthCredentials::ApiKey {
                key,
                value: resolved_value,
                location,
            },
            ..Default::default()
        });
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
    use crate::test_fixtures::{api_key_context, bearer_context, consumer_context};
    use connector::ApiKeyLocation;

    #[tokio::test]
    async fn resolves_plain_api_key() {
        let ctx = api_key_context("X-Api-Key", "my-key-value", ApiKeyLocation::Header).await;
        let result = ApiKeyAuthenticator.authenticate(&ctx).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let runtime = updated.runtime().expect("runtime must be set");
        match &runtime.auth {
            ResolvedAuthCredentials::ApiKey {
                key,
                value,
                location,
            } => {
                assert_eq!(key, "X-Api-Key");
                assert_eq!(value, "my-key-value");
                assert!(matches!(location, ApiKeyLocation::Header));
            }
            other => panic!("Expected ApiKey, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn resolves_query_param_api_key() {
        let ctx = api_key_context("api_key", "query-val", ApiKeyLocation::Query).await;
        let result = ApiKeyAuthenticator.authenticate(&ctx).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        match &updated.runtime().unwrap().auth {
            ResolvedAuthCredentials::ApiKey { location, .. } => {
                assert!(matches!(location, ApiKeyLocation::Query));
            }
            other => panic!("Expected ApiKey, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn returns_error_for_wrong_auth_type() {
        let ctx = bearer_context("token").await;
        let result = ApiKeyAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_error_without_connector() {
        let ctx = consumer_context().await;
        let result = ApiKeyAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }
}
