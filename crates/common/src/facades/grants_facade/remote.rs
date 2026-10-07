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

//! Adapter that reads grant tokens from the auth agent over HTTP.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::http::routes::fill;
use ymir::services::client::{ClientExt, ClientTrait};
use ymir::types::http::HttpBody;
use ymir::types::oauth::UserInfo;
use ymir::utils::{encode_url_safe_no_pad, http_client, ResponseExt};

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::grants_facade::{GrantsFacadeTrait, PeerToken, VerifiedPeer};
use crate::facades::VerifyTokenRequest;
use crate::routes::auth::{gate, peer_connection};

/// Grant tokens through the auth agent's API.
///
/// The calls carry no user token yet: the auth agent sees whoever its identity provider gives
/// (the fixed user with `static`; a 401 with `keycloak`). Propagating the caller is pending.
pub struct GrantsRemoteFacade {
    config: Arc<MinKnownConfig>,
}

impl GrantsRemoteFacade {
    pub fn new(config: Arc<MinKnownConfig>) -> Self {
        Self { config }
    }

    /// The auth agent's API, under its version.
    fn base_url(&self) -> String {
        format!(
            "{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_version()
        )
    }

    fn token_url(&self, route: &str, participant_id: &str, requested: bool) -> String {
        let segment = encode_url_safe_no_pad(participant_id);
        format!(
            "{}{}{}?requested={}",
            self.base_url(),
            peer_connection::PREFIX,
            fill(route, &segment),
            requested
        )
    }
}

#[async_trait]
impl GrantsFacadeTrait for GrantsRemoteFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn verify_token(&self, token: String) -> Outcome<VerifiedPeer> {
        let url = format!("{}{}{}", self.base_url(), gate::PREFIX, gate::TOKEN_VERIFY);
        http_client()
            .post_json::<VerifyTokenRequest, VerifiedPeer>(&url, None, &VerifyTokenRequest { token })
            .await
    }

    /// `participant_id` is the plain DID; it travels in base64url, as the auth agent expects it
    /// in the path. `user` does not travel yet (see the type's doc): the auth agent answers for
    /// its own caller. `200` carries the token and `202` means it is still pending.
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn peer_token(
        &self,
        _user: &UserInfo,
        participant_id: String,
        requested: bool,
    ) -> Outcome<PeerToken> {
        let url = self.token_url(peer_connection::TOKEN, &participant_id, requested);
        let response = http_client()
            .get(&url, None)
            .await?
            .ensure_success()
            .await?;
        if response.status() == StatusCode::ACCEPTED {
            return Ok(PeerToken::Pending);
        }
        Ok(PeerToken::Ready(response.parse_json().await?))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn refresh_peer_token(
        &self,
        _user: &UserInfo,
        participant_id: String,
        requested: bool,
    ) -> Outcome<PeerToken> {
        let url = self.token_url(peer_connection::TOKEN_REFRESH, &participant_id, requested);
        let response = http_client()
            .post(&url, None, HttpBody::None)
            .await?
            .ensure_success()
            .await?;
        if response.status() == StatusCode::ACCEPTED {
            return Ok(PeerToken::Pending);
        }
        Ok(PeerToken::Ready(response.parse_json().await?))
    }
}
