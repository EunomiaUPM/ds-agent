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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Adapter that registers datasets with the auth agent over HTTP.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use ymir::config::types::HostType;
use ymir::data::entities::shared::registered_dataset::Model as RegisteredDataset;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::types::http::HttpBody;
use ymir::utils::http_client;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::registry_facade::RegistryFacadeTrait;
use crate::routes::auth::gate;

/// Registration through the auth agent's API; like the other auth facades, without a user token.
pub struct RegistryRemoteFacade {
    config: Arc<MinKnownConfig>,
}

impl RegistryRemoteFacade {
    pub fn new(config: Arc<MinKnownConfig>) -> Self {
        Self { config }
    }

    fn url(&self, suffix: &str) -> String {
        format!(
            "{}{}{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_version(),
            gate::PREFIX,
            suffix
        )
    }
}

#[async_trait]
impl RegistryFacadeTrait for RegistryRemoteFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn sync_datasets(&self, datasets: Vec<RegisteredDataset>) -> Outcome<()> {
        let url = self.url(gate::REGISTRY_DATASETS);
        http_client()
            .put_json::<_, Value>(&url, None, &datasets)
            .await
            .map(|_| ())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth", count = datasets.len()))]
    async fn upsert_datasets(&self, datasets: Vec<RegisteredDataset>) -> Outcome<()> {
        if datasets.is_empty() {
            return Ok(());
        }
        let url = self.url(gate::REGISTRY_DATASETS);
        http_client()
            .post_ok(&url, None, HttpBody::json(&datasets)?)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth", count = ids.len()))]
    async fn unregister_datasets(&self, ids: Vec<String>) -> Outcome<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let url = self.url(gate::REGISTRY_DATASETS_REMOVE);
        http_client()
            .post_ok(&url, None, HttpBody::json(&ids)?)
            .await
    }
}
