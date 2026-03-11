use crate::entities::resource::ProtocolSpec;
use serde::{Deserialize, Serialize};

/// Pull lifecycle: a single protocol spec used to fetch data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullLifecycle {
    pub data_access: ProtocolSpec,
}
