/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::entities::peer_catalogs::PeerCatalogTrait;
use crate::protocols::dsp::types::catalog_definition::Catalog;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct PeerCatalogEntities {
    cache: Arc<dyn CatalogAgentCacheTrait>,
}

impl PeerCatalogEntities {
    pub(crate) fn new(cache: Arc<dyn CatalogAgentCacheTrait>) -> Self {
        PeerCatalogEntities { cache }
    }
}

#[async_trait::async_trait]
impl PeerCatalogTrait for PeerCatalogEntities {
    async fn get_peer_catalog(&self, peer_id: &String) -> Outcome<Option<Catalog>> {
        let peer_catalog = self.cache.get_peer_catalog_cache().get_catalog(peer_id).await;
        peer_catalog
    }

    async fn set_peer_catalog(&self, peer_id: &String, catalog: &Catalog) -> Outcome<()> {
        let peer_catalog = self.cache.get_peer_catalog_cache().set_catalog(peer_id, catalog).await;
        peer_catalog
    }
}
