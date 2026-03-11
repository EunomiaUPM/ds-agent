//! Interaction lifecycle types.
//!
//! An interaction describes *how* a connector exchanges data with its counterpart:
//!
//! - **Pull** — the connector actively fetches data on a schedule or on demand.
//!   It exposes a single `data_access` protocol spec.
//! - **Push** — the connector registers a callback endpoint with the remote side
//!   so that data is delivered asynchronously.  It has a `subscribe` spec for
//!   registration and an optional `unsubscribe` spec for deregistration.

use crate::entities::resource::ProtocolSpec;
use serde::{Deserialize, Serialize};

/// Top-level interaction mode of a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum InteractionConfig {
    #[serde(rename = "PULL")]
    Pull(PullLifecycle),

    #[serde(rename = "PUSH")]
    Push(PushLifecycle),
}

/// Pull lifecycle: a single protocol spec used to fetch data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullLifecycle {
    pub data_access: ProtocolSpec,
}

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
