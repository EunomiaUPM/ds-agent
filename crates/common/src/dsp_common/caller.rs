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

//! Who acts on a DSP endpoint, resolved once by the auth layers and read by the flow adapters.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::extract::{OriginalUri, Request, State};
use axum::http::{Extensions, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::http::OauthHttpMiddleware;
use ymir::types::oauth::UserInfo;

use crate::dsp_common::DspActor;
use crate::facades::grants_facade::{AccessNeed, GrantsFacadeTrait, VerifiedPeer};
use crate::facades::mates_facade::MatesFacadeTrait;
use crate::tck::TckPeer;

/// Who acts on a DSP process.
#[derive(Debug, Clone)]
pub enum DspCaller {
    /// A peer over the protocol: its verified token and, when known, its participant record.
    Peer {
        peer: VerifiedPeer,
        mate: Option<Mates>,
    },
    /// A local user over RPC, or the agent itself.
    User(UserInfo),
}

impl DspCaller {
    /// What the auth layers left: the caller `PeerAuth` put, or the user OAuth resolved.
    pub fn from_extensions(extensions: &Extensions) -> Outcome<Self> {
        if let Some(caller) = extensions.get::<Self>() {
            return Ok(caller.clone());
        }
        extensions
            .get::<UserInfo>()
            .cloned()
            .map(Self::User)
            .ok_or_else(|| Errors::crazy("no auth layer resolved the caller", None))
    }

    pub fn peer(&self) -> Outcome<&VerifiedPeer> {
        match self {
            Self::Peer { peer, .. } => Ok(peer),
            Self::User(_) => Err(Errors::crazy("a peer endpoint was called by a user", None)),
        }
    }

    pub fn user(&self) -> Outcome<&UserInfo> {
        match self {
            Self::User(user) => Ok(user),
            Self::Peer { .. } => Err(Errors::crazy("a user endpoint was called by a peer", None)),
        }
    }

    /// The caller as the actor a process authorizes.
    pub fn actor(&self) -> DspActor {
        match self {
            Self::Peer { peer, .. } => DspActor::peer(peer),
            Self::User(user) => DspActor::user(user),
        }
    }
}

/// Axum layer authenticating the peer on DSP routes: its GNAP token, then its participant.
#[derive(Clone)]
pub struct PeerAuth {
    grants: Arc<dyn GrantsFacadeTrait>,
    mates: Arc<dyn MatesFacadeTrait>,
    rejection: StatusCode,
    tck: bool,
}

impl PeerAuth {
    /// Rejects like a missing process (DSP 10.1.2.3), so probing cannot reveal which exist;
    /// in TCK mode it also accepts the TCK's token.
    pub fn new(grants: Arc<dyn GrantsFacadeTrait>, mates: Arc<dyn MatesFacadeTrait>) -> Self {
        Self {
            grants,
            mates,
            rejection: StatusCode::NOT_FOUND,
            tck: TckPeer::enabled(),
        }
    }

    /// Accepts the TCK's token or not, whatever the environment says.
    pub fn tck_mode(mut self, enabled: bool) -> Self {
        self.tck = enabled;
        self
    }

    /// Answers `status` instead to a missing or rejected token.
    pub fn rejecting_with(mut self, status: StatusCode) -> Self {
        self.rejection = status;
        self
    }

    /// Puts the `DspCaller` and its `VerifiedPeer` in the request; an unknown mate is no error.
    pub async fn run(State(auth): State<Self>, request: Request, next: Next) -> Response {
        let Ok(token) = OauthHttpMiddleware::bearer(request.headers()).map(str::to_owned) else {
            return auth.rejection.into_response();
        };
        let (mut request, need) = match Self::need(request).await {
            Ok(read) => read,
            Err(_) => return auth.rejection.into_response(),
        };
        let peer = match TckPeer::verify(auth.tck, &token) {
            Some(tck) => tck,
            None => {
                let verified = match need {
                    Some(need) => auth.grants.authorize(token, need).await,
                    None => auth.grants.verify_token(token).await,
                };
                match verified {
                    Ok(peer) => peer,
                    Err(_) => return auth.rejection.into_response(),
                }
            }
        };
        let mate = auth
            .mates
            .get_mate_by_id(&UserInfo::system(), peer.participant_id.clone())
            .await
            .inspect_err(|e| tracing::debug!(peer = %peer.participant_id, error = %e, "peer is not a known mate"))
            .ok();
        request.extensions_mut().insert(peer.clone());
        request
            .extensions_mut()
            .insert(DspCaller::Peer { peer, mate });
        next.run(request).await
    }

    /// What the endpoint gives access to, from its path (and the offer target of N1).
    async fn need(request: Request) -> Outcome<(Request, Option<AccessNeed>)> {
        let path = request
            .extensions()
            .get::<OriginalUri>()
            .map_or_else(|| request.uri().path(), |uri| uri.path())
            .to_string();
        if let Some((_, id)) = path.split_once("/catalog/datasets/") {
            return Ok((request, Some(AccessNeed::of_call(&path, Some(id), false))));
        }
        if path.ends_with("/catalog/request") || path.ends_with("/negotiations/offers") {
            return Ok((request, Some(AccessNeed::of_call(&path, None, false))));
        }
        if !path.ends_with("/negotiations/request") {
            return Ok((request, None));
        }
        let (parts, body) = request.into_parts();
        let bytes = to_bytes(body, OPENING_BODY_LIMIT)
            .await
            .map_err(|e| Errors::format(BadFormat::Received, e.to_string(), None))?;
        let target = serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|body| body["offer"]["target"].as_str().map(str::to_string));
        let need = target.map(|target| AccessNeed::of_call(&path, Some(&target), false));
        Ok((Request::from_parts(parts, Body::from(bytes)), need))
    }
}

const OPENING_BODY_LIMIT: usize = 1024 * 1024;
