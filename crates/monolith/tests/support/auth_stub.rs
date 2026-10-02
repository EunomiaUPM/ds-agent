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

//! Peer authentication without wallet or authority: each participant knows the other, and DSP
//! calls carry a fixed token per direction.

use std::sync::Arc;

use chrono::Utc;
use common::auth::{Claims, OauthTokenValidator, RbacRole};
use common::facades::mates_facade::MatesFacadeTrait;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use common::facades::AuthPorts;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{Errors, Outcome};
use ymir::types::participants::ParticipantType;

/// What one participant knows: itself, its peer and the token that peer must present.
pub struct StubAuth {
    me: Mates,
    peer: Mates,
    /// Token a call from the peer carries; the peer's record holds the one we present.
    inbound_token: String,
}

impl StubAuth {
    /// Ports where `me` knows `peer`, sends `outbound_token` and accepts `inbound_token`.
    pub fn ports(
        tenant: &str,
        me: (&str, &str),
        peer: (&str, &str),
        outbound_token: &str,
        inbound_token: &str,
    ) -> AuthPorts {
        let stub = Arc::new(Self {
            me: Self::mate(tenant, me.0, me.1, None),
            peer: Self::mate(tenant, peer.0, peer.1, Some(outbound_token)),
            inbound_token: inbound_token.to_string(),
        });
        AuthPorts {
            mates: stub.clone(),
            ssi_auth: stub,
        }
    }

    fn mate(tenant: &str, did: &str, base_url: &str, token: Option<&str>) -> Mates {
        Mates {
            tenant_id: tenant.to_string(),
            participant_id: did.to_string(),
            participant_nick: did.to_string(),
            participant_type: ParticipantType::Agent,
            base_url: base_url.to_string(),
            token: token.map(str::to_string),
            saved_at: Utc::now(),
            last_interaction: Utc::now(),
            extra_fields: serde_json::json!({}),
        }
    }

    fn for_tenant(&self, mate: &Mates, tenant: &str) -> Mates {
        Mates {
            tenant_id: tenant.to_string(),
            ..mate.clone()
        }
    }
}

#[async_trait::async_trait]
impl MatesFacadeTrait for StubAuth {
    async fn get_mate_by_id(&self, tenant_id: String, mate_id: String) -> Outcome<Mates> {
        if mate_id == self.peer.participant_id {
            Ok(self.for_tenant(&self.peer, &tenant_id))
        } else if mate_id == self.me.participant_id {
            Ok(self.for_tenant(&self.me, &tenant_id))
        } else {
            Err(Errors::missing_resource(
                mate_id,
                "unknown participant",
                None,
            ))
        }
    }

    async fn get_me_mate(&self, tenant_id: String) -> Outcome<Mates> {
        Ok(self.for_tenant(&self.me, &tenant_id))
    }

    async fn get_all_mates(&self, tenant_id: String) -> Outcome<Vec<Mates>> {
        Ok(vec![self.for_tenant(&self.peer, &tenant_id)])
    }
}

#[async_trait::async_trait]
impl SSIAuthFacadeTrait for StubAuth {
    async fn verify_token(&self, token: String) -> Outcome<Mates> {
        if token == self.inbound_token {
            Ok(self.peer.clone())
        } else {
            Err(Errors::unauthorized("unknown peer token", None))
        }
    }
}

/// Management API validator: the token `owner` is the owner of the participant's tenant.
pub struct OwnerValidator {
    pub tenant: String,
}

#[async_trait::async_trait]
impl OauthTokenValidator for OwnerValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        if token != "owner" {
            return Err(Errors::unauthorized("invalid token", None));
        }
        Ok(Claims {
            sub: self.tenant.clone(),
            role: RbacRole::Owner,
            iat: 0,
            exp: 9_999_999_999,
        })
    }
}
