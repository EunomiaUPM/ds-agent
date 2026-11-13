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

//! Short-TTL in-process cache of verified peer grant tokens and granted scopes.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use common::facades::grants_facade::VerifiedPeer;
use ymir::data::entities::received::grant;
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::access::ResourceAccess;
use ymir::utils::require_field;

/// Cached authorization snapshot for an approved bearer token hash.
#[derive(Clone)]
pub struct CachedGrantAuth {
    pub grant_id: String,
    pub peer: VerifiedPeer,
    pub granted: Option<Vec<ResourceAccess>>,
    token_expires_at: Option<DateTime<Utc>>,
    cached_at: Instant,
}

#[derive(Default)]
struct CacheState {
    by_hash: HashMap<String, CachedGrantAuth>,
    grant_to_hash: HashMap<String, String>,
}

/// Bounded L1 cache of validated GNAP tokens, invalidated locally on grant mutations.
pub struct TokenAuthCache {
    state: RwLock<CacheState>,
    ttl: Duration,
}

impl Default for TokenAuthCache {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TTL)
    }
}

impl TokenAuthCache {
    pub const DEFAULT_TTL: Duration = Duration::from_secs(5);
    const MAX_ENTRIES: usize = 4096;

    pub fn new(ttl: Duration) -> Self {
        Self {
            state: RwLock::new(CacheState::default()),
            ttl,
        }
    }

    /// Maps an approved grant model to its caller-facing [`VerifiedPeer`].
    pub fn verified_peer(grant: &grant::Model) -> Outcome<VerifiedPeer> {
        let participant_id = require_field(grant.participant_id.as_ref(), "participant_id")?;
        Ok(VerifiedPeer {
            participant_id: participant_id.to_string(),
            role: grant.role.clone(),
            visibility: grant.visibility.clone(),
        })
    }

    pub fn get_peer(&self, token_hash: &str) -> Option<VerifiedPeer> {
        self.read_valid(token_hash).map(|entry| entry.peer)
    }

    pub fn get_authorized(&self, token_hash: &str) -> Option<(String, VerifiedPeer, Vec<ResourceAccess>)> {
        let entry = self.read_valid(token_hash)?;
        let granted = entry.granted?;
        Some((entry.grant_id, entry.peer, granted))
    }

    pub fn put_peer(&self, token_hash: String, grant: &grant::Model, peer: &VerifiedPeer) {
        self.insert(
            token_hash,
            grant.id.clone(),
            peer.clone(),
            None,
            grant.final_expires_at,
        );
    }

    pub fn put_authorized(
        &self,
        token_hash: String,
        grant: &grant::Model,
        peer: VerifiedPeer,
        granted: Vec<ResourceAccess>,
    ) {
        self.insert(
            token_hash,
            grant.id.clone(),
            peer,
            Some(granted),
            grant.final_expires_at,
        );
    }

    pub fn invalidate_grant(&self, grant_id: &str) {
        if let Ok(mut guard) = self.state.write() {
            if let Some(hash) = guard.grant_to_hash.remove(grant_id) {
                guard.by_hash.remove(&hash);
            }
        }
    }

    fn read_valid(&self, token_hash: &str) -> Option<CachedGrantAuth> {
        let guard = self.state.read().ok()?;
        let entry = guard.by_hash.get(token_hash)?;
        let now = Utc::now();
        let valid_ttl = entry.cached_at.elapsed() < self.ttl;
        let not_expired = entry.token_expires_at.is_none_or(|exp| exp > now);
        (valid_ttl && not_expired).then(|| entry.clone())
    }

    fn insert(
        &self,
        token_hash: String,
        grant_id: String,
        peer: VerifiedPeer,
        granted: Option<Vec<ResourceAccess>>,
        token_expires_at: Option<DateTime<Utc>>,
    ) {
        if let Ok(mut guard) = self.state.write() {
            if guard.by_hash.len() >= Self::MAX_ENTRIES {
                let ttl = self.ttl;
                let now = Utc::now();
                let state = &mut *guard;
                state.by_hash.retain(|_, v| {
                    v.cached_at.elapsed() < ttl && v.token_expires_at.is_none_or(|exp| exp > now)
                });
                state.grant_to_hash.retain(|_, h| state.by_hash.contains_key(h));
                if guard.by_hash.len() >= Self::MAX_ENTRIES {
                    guard.by_hash.clear();
                    guard.grant_to_hash.clear();
                }
            }
            if let Some(old_hash) = guard.grant_to_hash.insert(grant_id.clone(), token_hash.clone()) {
                if old_hash != token_hash {
                    guard.by_hash.remove(&old_hash);
                }
            }
            let merged_granted = granted.or_else(|| {
                guard
                    .by_hash
                    .get(&token_hash)
                    .and_then(|existing| existing.granted.clone())
            });
            guard.by_hash.insert(
                token_hash,
                CachedGrantAuth {
                    grant_id,
                    peer,
                    granted: merged_granted,
                    token_expires_at,
                    cached_at: Instant::now(),
                },
            );
        }
    }
}
