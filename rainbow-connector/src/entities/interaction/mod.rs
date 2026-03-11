//! Interaction lifecycle types.
//!
//! An interaction describes *how* a connector exchanges data with its counterpart:
//!
//! - **Pull** — the connector actively fetches data on a schedule or on demand.
//!   It exposes a single `data_access` protocol spec.
//! - **Push** — the connector registers a callback endpoint with the remote side
//!   so that data is delivered asynchronously.  It has a `subscribe` spec for
//!   registration and an optional `unsubscribe` spec for deregistration.

pub mod pull;
pub mod push;

pub use pull::*;
pub use push::*;

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
