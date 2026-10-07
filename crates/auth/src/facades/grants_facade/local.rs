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

//! In-process adapter over the gatekeeper (received grants) and the peer connector (sent
//! grants).

use std::sync::Arc;

use async_trait::async_trait;
use common::facades::grants_facade::{GrantsFacadeTrait, PeerToken, VerifiedPeer};
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::modules::{GateKeeperModule, PeerConnectorModule};

/// Grant tokens read straight from the auth modules.
pub struct GrantsLocalFacade {
    gatekeeper: Arc<dyn GateKeeperModule>,
    peer_connector: Arc<dyn PeerConnectorModule>,
}

impl GrantsLocalFacade {
    pub fn new(
        gatekeeper: Arc<dyn GateKeeperModule>,
        peer_connector: Arc<dyn PeerConnectorModule>,
    ) -> Self {
        Self {
            gatekeeper,
            peer_connector,
        }
    }
}

#[async_trait]
impl GrantsFacadeTrait for GrantsLocalFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn verify_token(&self, token: String) -> Outcome<VerifiedPeer> {
        self.gatekeeper.verify_token(&token).await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "auth", user = %user.id())
    )]
    async fn peer_token(
        &self,
        user: &UserInfo,
        participant_id: String,
        requested: bool,
    ) -> Outcome<PeerToken> {
        self.peer_connector
            .peer_token(user, &participant_id, requested)
            .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "auth", user = %user.id())
    )]
    async fn refresh_peer_token(
        &self,
        user: &UserInfo,
        participant_id: String,
        requested: bool,
    ) -> Outcome<PeerToken> {
        self.peer_connector
            .refresh_peer_token(user, &participant_id, requested)
            .await
    }
}
