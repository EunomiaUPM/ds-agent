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

use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::types::oauth::{RolePath, UserInfo};
use ymir::types::participants::Visibility;
use ymir::utils::bearer_headers;

use crate::oauth::Owner;

pub mod remote;

/// Who a verified peer token belongs to, as the DSP endpoints need it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiedPeer {
    /// The peer's participant id (its DID).
    pub participant_id: String,
    /// Role that handles what the peer opens with this token (e.g. an inbound negotiation).
    pub role: RolePath,
    /// Who else sees what the peer opens, as its grant says.
    #[serde(default = "public")]
    pub visibility: Visibility,
}

fn public() -> Visibility {
    Visibility::Public
}

impl VerifiedPeer {
    /// Owner of what the peer opens: nobody, handled by the role of its grant and seen as the
    /// grant says.
    pub fn owner(&self) -> Owner {
        Owner::team(self.role.clone(), self.visibility.clone())
    }

    /// The peer as an actor of the agents: its DID as user id, under the role of its grant. It
    /// then reads and acts with the same rules as a local user of that role.
    pub fn to_user(&self) -> UserInfo {
        UserInfo::new(
            self.participant_id.clone(),
            None,
            self.role.clone(),
            serde_json::Map::new(),
        )
    }
}

#[mockall::automock]
#[async_trait]
/// Tokens of the GNAP grants with peers, for the DSP endpoints and calls.
pub trait GrantsFacadeTrait: Send + Sync {
    /// The peer behind `token`, if this connector issued it and the grant is approved; fails
    /// otherwise (the DSP endpoint then rejects the call).
    async fn verify_token(&self, token: String) -> Outcome<VerifiedPeer>;

    /// The token `user` presents to `participant_id`: the one of its own approved grant with that
    /// peer, even if a colleague has another. Without one, a grant is requested on the user's
    /// behalf and the answer is `Pending` until it completes. `requested` says whether the grant
    /// is for something this connector asks for (`true`) or answers (`false`).
    async fn peer_token(
        &self,
        user: &UserInfo,
        participant_id: String,
        requested: bool,
    ) -> Outcome<PeerToken>;

    async fn refresh_peer_token(
        &self,
        user: &UserInfo,
        participant_id: String,
        requested: bool,
    ) -> Outcome<PeerToken>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerToken {
    Ready(String),
    Pending,
}

pub const PEER_TOKEN_WAIT: Duration = Duration::from_secs(20);
pub const PEER_TOKEN_POLL: Duration = Duration::from_secs(1);

pub async fn obtain_peer_token(
    grants: &dyn GrantsFacadeTrait,
    user: &UserInfo,
    participant_id: &str,
    requested: bool,
) -> Outcome<Option<String>> {
    let first = grants
        .peer_token(user, participant_id.to_string(), requested)
        .await?;
    wait_peer_token(grants, user, participant_id, requested, first).await
}

async fn wait_peer_token(
    grants: &dyn GrantsFacadeTrait,
    user: &UserInfo,
    participant_id: &str,
    requested: bool,
    first: PeerToken,
) -> Outcome<Option<String>> {
    let deadline = Instant::now() + PEER_TOKEN_WAIT;
    let mut state = first;
    loop {
        match state {
            PeerToken::Ready(token) => return Ok(Some(token)),
            PeerToken::Pending if Instant::now() + PEER_TOKEN_POLL <= deadline => {
                tokio::time::sleep(PEER_TOKEN_POLL).await;
                state = grants
                    .peer_token(user, participant_id.to_string(), requested)
                    .await?;
            }
            PeerToken::Pending => return Ok(None),
        }
    }
}

pub async fn send_with_peer_token<T, F, Fut>(
    grants: &dyn GrantsFacadeTrait,
    user: &UserInfo,
    participant_id: &str,
    requested: bool,
    mut send: F,
) -> Outcome<T>
where
    F: FnMut(Option<HeaderMap>) -> Fut,
    Fut: Future<Output = Outcome<T>>,
{
    let token = match obtain_peer_token(grants, user, participant_id, requested).await {
        Ok(token) => token,
        Err(e) => {
            tracing::warn!("No token towards {participant_id}, sending without one: {e}");
            None
        }
    };
    let headers = token.as_deref().map(bearer_headers).transpose()?;
    let result = send(headers).await;
    if !matches!(&result, Err(e) if is_unauthorized(e)) {
        return result;
    }

    let refreshed = match grants
        .refresh_peer_token(user, participant_id.to_string(), requested)
        .await
    {
        Ok(first) => wait_peer_token(grants, user, participant_id, requested, first).await,
        Err(e) => Err(e),
    };
    match refreshed {
        Ok(Some(token)) => send(Some(bearer_headers(&token)?)).await,
        Ok(None) => result,
        Err(e) => {
            tracing::warn!("Could not refresh the token towards {participant_id}: {e}");
            result
        }
    }
}

fn is_unauthorized(error: &Errors) -> bool {
    matches!(
        error,
        Errors::PetitionError {
            failure: PetitionFailure::HttpStatus(StatusCode::UNAUTHORIZED),
            ..
        }
    )
}
