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

//! The party acting on a DSP process: a remote peer over the protocol or a local user over RPC.

use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::facades::grants_facade::VerifiedPeer;
use crate::oauth::{Owner, OwnershipTrait};

/// Who is acting on a DSP process, as established by authentication.
#[derive(Debug, Clone)]
pub enum DspActor {
    /// A remote connector authenticated through its GNAP token.
    Peer { participant_id: String },
    /// A local user authenticated through OAuth.
    User(UserInfo),
}

impl DspActor {
    /// Peer whose token the grants facade verified.
    pub fn peer(peer: &VerifiedPeer) -> Self {
        Self::Peer {
            participant_id: peer.participant_id.clone(),
        }
    }

    /// Local user authenticated through OAuth.
    pub fn user(user: &UserInfo) -> Self {
        Self::User(user.clone())
    }

    /// A peer may only act on processes where it is the `counterparty`; a user only on processes
    /// it [acts on](OwnershipTrait::acts_on): its own, those below its role, and those a peer
    /// opened for its very role. A refusal looks like a missing process.
    pub fn authorize(&self, owner: &Owner, counterparty: &str, pid: &Urn) -> Outcome<()> {
        let allowed = match self {
            Self::Peer { participant_id } => counterparty == participant_id,
            Self::User(user) => user.acts_on_owner(owner),
        };
        if allowed {
            Ok(())
        } else {
            Err(Errors::missing_resource(
                pid.to_string(),
                "process not found",
                None,
            ))
        }
    }
}
