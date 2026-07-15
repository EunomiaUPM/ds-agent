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

use oauth::entities::user::User;
use ymir::data::entities::shared::participant::Model as Mates;

/// Authentication info por Context

/// Common view over the auth attached to any transfer context, whatever the
/// source. Lets the shared context / command code read identity and token
/// without knowing whether it came from an inbound DSP peer or an outbound RPC
/// caller.
pub trait TransferAuthn {
    fn raw(&self) -> &str;
    fn token_type(&self) -> &str;
    fn token_content(&self) -> &str;
    fn participant(&self) -> &Mates;
}

/// Inbound DSP: a peer authenticated to us. `associated_participant` is that
/// remote peer, resolved by the auth middleware before we ran.
#[derive(Debug)]
pub struct TransferDSPAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub associated_participant: Mates,
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
    fn participant(&self) -> &Mates {
        &self.associated_participant
    }
}

/// Outbound RPC: our own app driving a transfer. `me_participant` is us, and
/// `me_user` is the acting user behind the RPC call.
#[derive(Debug)]
pub struct TransferRPCAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub me_participant: Mates,
    pub me_user: User,
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
    fn participant(&self) -> &Mates {
        &self.me_participant
    }
}
