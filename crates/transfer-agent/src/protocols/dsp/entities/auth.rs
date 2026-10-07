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

//! Who authenticated on a transfer context: an inbound DSP peer, or the user
//! behind an outbound RPC call.

use common::facades::grants_facade::VerifiedPeer;
use common::oauth::UserInfo;

/// The auth on any transfer context, whatever the source. Lets shared code read
/// identity and token without knowing which side authenticated.
pub trait TransferAuthn {
    /// Full `Authorization` header as received.
    fn raw(&self) -> &str;
    /// Scheme part of the header, such as `Bearer`.
    fn token_type(&self) -> &str;
    /// Token part of the header.
    fn token_content(&self) -> &str;
    // Participant behind the token: unused, and with identity on `UserInfo` the RPC side has no
    // participant record of its own. Kept as it was.
    // fn participant(&self) -> &Mates;
}

/// Inbound DSP: a peer authenticated to us. `associated_participant` is that
/// remote peer, resolved by the auth middleware before we ran.
#[derive(Debug)]
pub struct TransferDSPAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub associated_participant: VerifiedPeer,
}

impl TransferAuthn for TransferDSPAuthn {
    fn raw(&self) -> &str {
        &self.raw
    }
    fn token_type(&self) -> &str {
        &self.token_type
    }
    fn token_content(&self) -> &str {
        &self.token_content
    }
    // fn participant(&self) -> &Mates {
    //     &self.associated_participant
    // }
}

/// Outbound RPC: our own app driving a transfer. `me_user` is the acting user behind the RPC
/// call, as the OAuth middleware resolved it.
#[derive(Debug)]
pub struct TransferRPCAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub me_user: UserInfo,
}

impl TransferAuthn for TransferRPCAuthn {
    fn raw(&self) -> &str {
        &self.raw
    }
    fn token_type(&self) -> &str {
        &self.token_type
    }
    fn token_content(&self) -> &str {
        &self.token_content
    }
    // fn participant(&self) -> &Mates {
    //     &self.me_participant
    // }
}
