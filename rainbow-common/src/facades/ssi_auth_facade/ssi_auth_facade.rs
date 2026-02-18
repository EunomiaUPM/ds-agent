/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::config::services::MinKnownConfig;
use crate::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use crate::http_client::HttpClient;
use crate::mates::mates::VerifyTokenRequest;
use crate::mates::Mates;
use async_trait::async_trait;
use std::sync::Arc;
use ymir::config::types::HostType;

const SSI_AUTH_FACADE_VERIFICATION_URL: &str = "/api/v1/mates/token";

pub struct SSIAuthFacadeService {
    config: Arc<MinKnownConfig>,
    client: Arc<HttpClient>,
}

impl SSIAuthFacadeService {
    pub fn new(config: Arc<MinKnownConfig>, client: Arc<HttpClient>) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl SSIAuthFacadeTrait for SSIAuthFacadeService {
    async fn verify_token(&self, token: String) -> anyhow::Result<Mates> {
        let base_url = self.config.get_host(HostType::Http);
        let url = format!("{}{}", base_url, SSI_AUTH_FACADE_VERIFICATION_URL);
        let mate = self
            .client
            .post_json::<VerifyTokenRequest, Mates>(url.as_str(), &VerifyTokenRequest { token })
            .await?;
        Ok(mate)
    }
}
