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
pub struct BasicConfigAuthenticator;

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for BasicConfigAuthenticator {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "data-endpoint"))]
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or_else(|| DataplaneError::ConnectorNotAvailable)?;

        let AuthenticationConfig::BasicAuth(basic) = connector.authentication_config.clone() else {
            return Err(DataplaneError::AuthConfigMismatch {
                expected: "BasicAuth".to_string(),
            }
            .into());
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
