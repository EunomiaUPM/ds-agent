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

use std::sync::Arc;

use async_trait::async_trait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use crate::auth::ServiceHttpClient;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use crate::facades::VerifyTokenRequest;
use ymir::data::entities::shared::participant::Model as Mates;

const SSI_AUTH_FACADE_VERIFICATION_URL: &str = "/api/v1/mates/token";

pub struct SSIAuthRemoteFacade {
    config: Arc<MinKnownConfig>,
    client: Arc<ServiceHttpClient>,
}

impl SSIAuthRemoteFacade {
    pub fn new(config: Arc<MinKnownConfig>, client: Arc<ServiceHttpClient>) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl SSIAuthFacadeTrait for SSIAuthRemoteFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn verify_token(&self, token: String) -> Outcome<Mates> {
        let base_url = self.config.get_host(HostType::Http);
        let url = format!("{}{}", base_url, SSI_AUTH_FACADE_VERIFICATION_URL);
        let mate = self
            .client
            .post_json::<VerifyTokenRequest, Mates>(
                url.as_str(),
                None,
                &VerifyTokenRequest { token },
            )
            .await?;
        Ok(mate)
    }
}
