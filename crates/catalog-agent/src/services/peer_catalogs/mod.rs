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
use common::auth::AccessScope;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait PeerCatalogServiceTrait: Send + Sync {
    async fn get_all_peer_catalogs(&self, scope: &AccessScope) -> Outcome<Vec<(Mates, Catalog)>>;
    async fn get_peer_catalog(
        &self,
        scope: &AccessScope,
        peer_id: &str,
    ) -> Outcome<Option<Catalog>>;
    async fn set_peer_catalog(
        &self,
        scope: &AccessScope,
        peer_id: &str,
        catalog: &Catalog,
    ) -> Outcome<()>;
}
