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

//! Adapter that reads participants from the auth agent over HTTP.

use std::sync::Arc;

use async_trait::async_trait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::mates_facade::MatesFacadeTrait;
use crate::paginated_spec::Paginated;
use ymir::data::entities::shared::participant::Model as Mates;

/// Paths of the auth agent's `/mates` API, under its API version.
const MATES_PATH: &str = "/mates";
const MATES_MYSELF_PATH: &str = "/mates/myself";
const MATES_ALL_PATH: &str = "/mates/all";

/// Participants read from the auth agent's `/mates` API.
pub struct MatesRemoteFacade {
    config: Arc<MinKnownConfig>,
}

impl MatesRemoteFacade {
    pub fn new(config: Arc<MinKnownConfig>) -> Self {
        Self { config }
    }

    fn base_url(&self) -> String {
        format!(
            "{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_version()
        )
    }
}

#[async_trait]
impl MatesFacadeTrait for MatesRemoteFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_mate_by_id(&self, mate_id: String) -> Outcome<Mates> {
        let url = format!("{}{}/{}", self.base_url(), MATES_PATH, mate_id);
        http_client().get_json(&url, None).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_me_mate(&self) -> Outcome<Mates> {
        let url = format!("{}{}", self.base_url(), MATES_MYSELF_PATH);
        http_client().get_json(&url, None).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_all_mates(&self) -> Outcome<Vec<Mates>> {
        let url = format!("{}{}", self.base_url(), MATES_ALL_PATH);
        let page: Paginated<Mates> = http_client().get_json(&url, None).await?;
        Ok(page.items)
    }
}
