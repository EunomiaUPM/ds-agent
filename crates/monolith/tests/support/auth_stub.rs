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
use common::facades::grants_facade::{GrantsFacadeTrait, VerifiedPeer};
use common::facades::mates_facade::MatesFacadeTrait;
use common::facades::AuthPorts;
use common::oauth::{OauthTokenValidatorTrait, RolePath, UserInfo, Visibility};
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{Errors, Outcome};
use ymir::types::participants::ParticipantType;

/// What one participant knows: itself, its peer, the token it presents to that peer and the
/// token that peer must present.
pub struct StubAuth {
    me: Mates,
    peer: Mates,
    /// Token a call to the peer carries (our grant with it).
    outbound_token: String,
    /// Token a call from the peer carries (the grant we gave it).
    inbound_token: String,
}

impl StubAuth {
    /// Ports where `me` knows `peer`, sends `outbound_token` and accepts `inbound_token`.
    pub fn ports(
        _tenant: &str,
        me: (&str, &str),
        peer: (&str, &str),
        outbound_token: &str,
        inbound_token: &str,
    ) -> AuthPorts {
        let stub = Arc::new(Self {
            me: Self::mate(me.0, me.1),
            peer: Self::mate(peer.0, peer.1),
            outbound_token: outbound_token.to_string(),
            inbound_token: inbound_token.to_string(),
        });
        AuthPorts {
            mates: stub.clone(),
            grants: stub,
        }
    }

    fn mate(did: &str, base_url: &str) -> Mates {
        Mates {
            participant_id: did.to_string(),
            participant_nick: did.to_string(),
            participant_type: ParticipantType::Agent,
            base_url: base_url.to_string(),
            saved_at: Utc::now(),
            last_interaction: Utc::now(),
            extra_fields: serde_json::json!({}),
        }
    }
}

#[async_trait::async_trait]
impl MatesFacadeTrait for StubAuth {
    async fn get_mate_by_id(&self, _user: &UserInfo, mate_id: String) -> Outcome<Mates> {
        if mate_id == self.peer.participant_id {
            Ok(self.peer.clone())
        } else if mate_id == self.me.participant_id {
            Ok(self.me.clone())
        } else {
            Err(Errors::missing_resource(
                mate_id,
                "unknown participant",
                None,
            ))
        }
    }

    async fn get_me_mate(&self) -> Outcome<Mates> {
        Ok(self.me.clone())
    }

    async fn get_all_mates(&self, _user: &UserInfo) -> Outcome<Vec<Mates>> {
        Ok(vec![self.peer.clone()])
    }
}

#[async_trait::async_trait]
impl GrantsFacadeTrait for StubAuth {
    async fn verify_token(&self, token: String) -> Outcome<VerifiedPeer> {
        if token == self.inbound_token {
            Ok(VerifiedPeer {
                participant_id: self.peer.participant_id.clone(),
                role: RolePath::root(),
                visibility: Visibility::Public,
            })
        } else {
            Err(Errors::unauthorized("unknown peer token", None))
        }
    }

    async fn peer_token(&self, _user: &UserInfo, participant_id: String) -> Outcome<Option<String>> {
        Ok((participant_id == self.peer.participant_id).then(|| self.outbound_token.clone()))
    }
}

/// Management API validator: the token `owner` is the root of the participant, acting with the
/// participant's tenant as its id (stage A: processes a peer opens belong to the root).
pub struct OwnerValidator {
    pub tenant: String,
}

#[async_trait::async_trait]
impl OauthTokenValidatorTrait for OwnerValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        if token != Some("owner") {
            return Err(Errors::unauthorized("invalid token", None));
        }
        Ok(UserInfo::new(
            self.tenant.clone(),
            None,
            RolePath::root(),
            serde_json::Map::new(),
        ))
    }
}
