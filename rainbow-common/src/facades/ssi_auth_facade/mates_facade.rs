/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use super::MatesFacadeTrait;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::http_client::HttpClient;
use crate::mates::Mates;
use async_trait::async_trait;
use std::sync::Arc;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

pub struct MatesFacadeService {
    config: Arc<MinKnownConfig>,
    client: Arc<HttpClient>,
}

impl MatesFacadeService {
    pub fn new(config: Arc<MinKnownConfig>, client: Arc<HttpClient>) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl MatesFacadeTrait for MatesFacadeService {
    async fn get_mate_by_id(&self, mate_id: String) -> Outcome<Mates> {
        let ssi_auth_url = self.config.get_host(HostType::Http);
        let mates_url = format!("{}/api/v1/mates/{}", ssi_auth_url, mate_id);
        let mates = self.client.get_json::<Mates>(mates_url.as_str()).await?;
        Ok(mates)
    }

    async fn get_mate_by_slug(&self, mate_slug: String) -> Outcome<Mates> {
        let ssi_auth_url = self.config.get_host(HostType::Http);
        let mates_url = format!("{}/api/v1/mates/slug/{}", ssi_auth_url, mate_slug);
        let mates = self.client.get_json::<Mates>(mates_url.as_str()).await?;
        Ok(mates)
    }

    async fn get_me_mate(&self) -> Outcome<Mates> {
        let ssi_auth_url = self.config.get_host(HostType::Http);
        let mates_url = format!("{}/api/v1/mates/myself", ssi_auth_url);
        let mates = self.client.get_json::<Mates>(mates_url.as_str()).await?;
        Ok(mates)
    }
}
