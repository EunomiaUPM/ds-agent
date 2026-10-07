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

//! Adapter that verifies tokens through the auth agent over HTTP.

use std::sync::Arc;

use async_trait::async_trait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use crate::facades::VerifyTokenRequest;
use ymir::data::entities::shared::participant::Model as Mates;

/// Path of the token verification under the auth agent's API version.
const SSI_AUTH_FACADE_VERIFICATION_PATH: &str = "/mates/token";

/// Token verification through the auth agent's API.
pub struct SSIAuthRemoteFacade {
    config: Arc<MinKnownConfig>,
}

impl SSIAuthRemoteFacade {
    pub fn new(config: Arc<MinKnownConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl SSIAuthFacadeTrait for SSIAuthRemoteFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn verify_token(&self, token: String) -> Outcome<Mates> {
        let url = format!(
            "{}{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_version(),
            SSI_AUTH_FACADE_VERIFICATION_PATH
        );
        let mate = http_client()
            .post_json::<VerifyTokenRequest, Mates>(url.as_str(), None, &VerifyTokenRequest { token })
            .await?;
        Ok(mate)
    }
}
