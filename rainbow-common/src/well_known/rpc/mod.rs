pub mod rpc;

use crate::dsp_common::well_known_types::{DSPProtocolVersions, VersionPath, VersionResponse};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

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
