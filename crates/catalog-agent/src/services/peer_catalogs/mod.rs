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

//! Catalogs fetched from peer connectors, kept in the cache as the peer sent them.

pub mod service;

use serde_json::Value;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

/// Catalogs fetched from peers, kept per user.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PeerCatalogServiceTrait: Send + Sync {
    /// Every cached peer catalog, with the peer it came from.
    async fn get_all(&self, user: &UserInfo) -> Outcome<Vec<(Mates, Value)>>;
    /// Cached catalog of the peer, if any.
    async fn get(&self, user: &UserInfo, peer_id: &str) -> Outcome<Option<Value>>;
    /// Caches the catalog fetched from the peer.
    async fn set(&self, user: &UserInfo, peer_id: &str, catalog: &Value) -> Outcome<()>;
}
