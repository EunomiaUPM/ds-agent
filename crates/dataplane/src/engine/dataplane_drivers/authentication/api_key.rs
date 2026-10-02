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

use crate::engine::dataplane_drivers::DriverAuthenticatorTrait;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use crate::errors::DataplaneError;
use connector::AuthenticationConfig;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct ApiKeyAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for ApiKeyAuthenticator {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "data-endpoint"))]
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or(DataplaneError::ConnectorNotAvailable)?;

        let AuthenticationConfig::ApiKey {
            key,
            value,
            location,
        } = connector.authentication_config.clone()
        else {
            return Err(DataplaneError::AuthConfigMismatch {
                expected: "ApiKey".to_string(),
            }
            .into());
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
