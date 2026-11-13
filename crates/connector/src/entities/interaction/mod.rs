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

//! Interaction lifecycle types.
//!
//! An interaction describes *how* a connector exchanges data with its counterpart:
//!
//! - **Pull** — the connector actively fetches data on a schedule or on demand. It exposes a single
//!   `data_access` protocol spec.
//! - **Push** — the connector registers a callback endpoint with the remote side so that data is
//!   delivered asynchronously.  It has a `subscribe` spec for registration and an optional
//!   `unsubscribe` spec for deregistration.

pub mod pull;
pub mod push;

pub use pull::*;
pub use push::*;

use crate::entities::resource::ProtocolSpec;
use serde::{Deserialize, Serialize};
use ymir::errors::{Errors, Outcome};

/// Maximum `maxRetries` accepted on an interaction lifecycle so misconfigurations cannot stall
/// transfers indefinitely.
pub const MAX_INTERACTION_RETRIES: u32 = 20;

/// Whether a transfer stays open indefinitely or auto-completes once fulfilled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransferLifetime {
    /// Open-ended channel until an explicit DSP completion or termination message arrives.
    #[default]
    Infinite,
    /// Single-shot / self-completing transfer: signals `Completed` once its delivery succeeds.
    Finite,
}

/// Top-level interaction mode of a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum InteractionConfig {
    #[serde(rename = "PULL")]
    Pull(PullLifecycle),

    #[serde(rename = "PUSH")]
    Push(PushLifecycle),
}

impl InteractionConfig {
    pub fn lifetime(&self) -> TransferLifetime {
        match self {
            Self::Pull(lc) => lc.lifetime,
            Self::Push(lc) => lc.lifetime,
        }
    }

    pub fn max_retries(&self) -> Option<u32> {
        match self {
            Self::Pull(lc) => lc.max_retries,
            Self::Push(lc) => lc.max_retries,
        }
    }

    pub fn validate(&self) -> Outcome<()> {
        if let Some(retries) = self.max_retries() {
            if retries > MAX_INTERACTION_RETRIES {
                return Err(Errors::validation(
                    format!(
                        "interaction maxRetries ({retries}) exceeds maximum allowed ({MAX_INTERACTION_RETRIES})"
                    ),
                    None,
                ));
            }
        }
        match self {
            Self::Pull(lc) => {
                if matches!(lc.data_access, ProtocolSpec::Kafka(_)) {
                    return Err(Errors::validation(
                        "KAFKA protocol is not supported in PULL mode; use PUSH mode (subscribe)",
                        None,
                    ));
                }
            }
            Self::Push(lc) => {
                if matches!(lc.unsubscribe, Some(ProtocolSpec::Kafka(_))) {
                    return Err(Errors::validation(
                        "KAFKA push interactions stop the managed consumer automatically; unsubscribe must be null",
                        None,
                    ));
                }
            }
        }
        Ok(())
    }
}
