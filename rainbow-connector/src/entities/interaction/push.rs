use crate::entities::resource::ProtocolSpec;
use serde::{Deserialize, Serialize};

/// Push lifecycle: a subscribe spec plus an optional unsubscribe spec.
///
/// After a successful subscribe call the remote side will push data to the
/// registered callback URL.  The `unsubscribe` spec, when present, is called
/// to deregister the callback.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushLifecycle {
    pub subscribe: ProtocolSpec,
    pub unsubscribe: Option<ProtocolSpec>,
}
