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

pub mod rpc;

use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

use crate::dsp_common::well_known_types::{DSPProtocolVersions, VersionPath, VersionResponse};

pub const DSP_CURRENT_VERSION: DSPProtocolVersions = DSPProtocolVersions::V2025_1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WellKnownRPCRequest {
    pub participant_id: String,
}
#[async_trait::async_trait]
pub trait WellKnownRPCTrait: Send + Sync {
    async fn fetch_dataspace_well_known(
        &self,
        input: &WellKnownRPCRequest,
    ) -> Outcome<(VersionResponse, String)>;
    async fn fetch_dataspace_current_path(
        &self,
        input: &WellKnownRPCRequest,
    ) -> Outcome<VersionPath>;
}
