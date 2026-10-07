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

//! Cached catalogs fetched from peer connectors.

pub mod service;

use crate::protocols::dsp::types::catalog_definition::Catalog;
use common::oauth::UserInfo;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

/// Catalogs fetched from peers, kept in the cache.
#[async_trait::async_trait]
pub trait PeerCatalogServiceTrait: Send + Sync {
    /// Every cached peer catalog, with the peer it came from.
    async fn get_all_peer_catalogs(&self, user: &UserInfo) -> Outcome<Vec<(Mates, Catalog)>>;
    /// Cached catalog of the peer, if any.
    async fn get_peer_catalog(
        &self,
        user: &UserInfo,
        peer_id: &str,
    ) -> Outcome<Option<Catalog>>;
    /// Caches the catalog fetched from the peer.
    async fn set_peer_catalog(
        &self,
        user: &UserInfo,
        peer_id: &str,
        catalog: &Catalog,
    ) -> Outcome<()>;
}
