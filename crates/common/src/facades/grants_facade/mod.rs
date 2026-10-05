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

//! GNAP access tokens between this connector and its peers, one per direction.
//!
//! A token a peer presents was issued by this connector's gatekeeper and lives in a received
//! grant; a token this connector presents to a peer was issued by that peer and lives in a sent
//! grant of the user who asked for it. A token is only valid in the direction it was issued.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;
use ymir::types::oauth::{RolePath, UserInfo};

pub mod remote;

/// Who a verified peer token belongs to, as the DSP endpoints need it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiedPeer {
    /// The peer's participant id (its DID).
    pub participant_id: String,
    /// Role that handles what the peer opens with this token (e.g. an inbound negotiation).
    pub role: RolePath,
}

#[mockall::automock]
#[async_trait]
/// Tokens of the GNAP grants with peers, for the DSP endpoints and calls.
pub trait GrantsFacadeTrait: Send + Sync {
    /// The peer behind `token`, if this connector issued it and the grant is approved; fails
    /// otherwise (the DSP endpoint then rejects the call).
    async fn verify_token(&self, token: String) -> Outcome<VerifiedPeer>;

    /// The token `user` presents to `participant_id`: the one of its own approved grant with that
    /// peer. `None` if the user has none, even if a colleague does.
    async fn peer_token(&self, user: &UserInfo, participant_id: String) -> Outcome<Option<String>>;
}
