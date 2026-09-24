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

use crate::protocols::dsp::facades::well_known_rpc_facade::WellKnownRPCFacadeTrait;
use common::config::services::CatalogConfig;
use common::config::types::traits::CommonConfigTrait;
use common::dsp_common::well_known_types::{Version, VersionPath};
use common::well_known::rpc::WellKnownRPCRequest;
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

const RPC_WELL_KNOWN_PATH: &str = "/rpc/.well-known/dspace-version/path";

pub struct WellKnownRPCFacadeForDSProtocol {
    config: Arc<CatalogConfig>,
}

impl WellKnownRPCFacadeForDSProtocol {
    pub fn new(config: Arc<CatalogConfig>) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl WellKnownRPCFacadeTrait for WellKnownRPCFacadeForDSProtocol {
    async fn resolve_dataspace_current_path(&self, input: &WellKnownRPCRequest) -> Outcome<String> {
        let host = self.config.common().get_host(HostType::Http);
        let url = format!("{}{}", host, RPC_WELL_KNOWN_PATH);
        let provider_address = http_client()
            .post_json::<WellKnownRPCRequest, VersionPath>(&url, None, input)
            .await?;
        Ok(provider_address.path)
    }
}
