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

use crate::auth::AccessScope;
use urn::Urn;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{Errors, Outcome};

/// Who is acting on a DSP process, as established by authentication.
#[derive(Debug, Clone)]
pub enum DspActor {
    /// A remote connector authenticated through the SSI token.
    Peer {
        tenant_id: String,
        participant_id: String,
    },
    /// A local user authenticated through OAuth.
    User(AccessScope),
}

impl DspActor {
    pub fn peer(mate: &Mates) -> Self {
        Self::Peer {
            tenant_id: mate.tenant_id.clone(),
            participant_id: mate.participant_id.clone(),
        }
    }

    pub fn user(scope: &AccessScope) -> Self {
        Self::User(scope.clone())
    }

    /// A peer may only act on processes of the tenant it onboarded into where it is the
    /// `counterparty`; a user only within its tenants. A refusal looks like a missing process.
    pub fn authorize(&self, owner_tenant: &str, counterparty: &str, pid: &Urn) -> Outcome<()> {
        let allowed = match self {
            Self::Peer {
                tenant_id,
                participant_id,
            } => owner_tenant == tenant_id && counterparty == participant_id,
            Self::User(scope) => scope.permits(owner_tenant),
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
