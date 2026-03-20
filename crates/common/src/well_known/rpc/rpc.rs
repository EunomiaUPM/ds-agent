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

use std::sync::Arc;

use ymir::errors::{Errors, Outcome};

use crate::dsp_common::well_known_types::{VersionPath, VersionResponse};
use crate::facades::ssi_auth_facade::MatesFacadeTrait;
use crate::http_client::HttpClient;
use crate::well_known::rpc::{WellKnownRPCRequest, WellKnownRPCTrait, DSP_CURRENT_VERSION};

pub struct WellKnownRPCService {
    http_client: Arc<HttpClient>,
    mates_facade: Arc<dyn MatesFacadeTrait>,
}

impl WellKnownRPCService {
    pub fn new(http_client: Arc<HttpClient>, mates_facade: Arc<dyn MatesFacadeTrait>) -> Self {
        Self {
            http_client,
            mates_facade,
        }
    }
    async fn get_base_url(&self, mate_id: &str) -> Outcome<String> {
        let participant = self
            .mates_facade
            .get_mate_by_id(mate_id.to_string())
            .await
            .map_err(|e| Errors::missing_resource(mate_id, "Mate not found", Some(Box::new(e))))?;
        participant
            .base_url
            .ok_or_else(|| Errors::missing_resource(mate_id, "Base url not found", None))
    }
}

#[async_trait::async_trait]
impl WellKnownRPCTrait for WellKnownRPCService {
    async fn fetch_dataspace_well_known(
        &self,
        input: &WellKnownRPCRequest,
    ) -> Outcome<(VersionResponse, String)> {
        let mate_id = input.participant_id.clone();
        let base_url = self.get_base_url(&mate_id).await?;
        let url = format!("{}/.well-known/dspace-version", base_url);
        let response = self
            .http_client
            .get_json::<VersionResponse>(url.as_str())
            .await?;
        Ok((response, base_url))
    }

    async fn fetch_dataspace_current_path(
        &self,
        input: &WellKnownRPCRequest,
    ) -> Outcome<VersionPath> {
        let (wk, base_url) = self.fetch_dataspace_well_known(input).await?;

        let current = wk
            .protocol_versions
            .iter()
            .find(|p| p.version == DSP_CURRENT_VERSION);
        if current.is_none() {
            return Err(Errors::parse("Could not find protocol version", None));
        }
        let current = current.unwrap();
        let path = format!("{}{}", base_url, current.path);
        Ok(VersionPath { path })
    }
}
