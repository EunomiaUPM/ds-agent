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
