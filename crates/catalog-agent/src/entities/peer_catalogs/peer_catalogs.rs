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

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::entities::peer_catalogs::PeerCatalogTrait;
use crate::protocols::dsp::types::catalog_definition::Catalog;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use std::sync::Arc;
use tracing::warn;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

pub struct PeerCatalogEntities {
    cache: Arc<dyn CatalogAgentCacheTrait>,
    mates_facade: Arc<dyn MatesFacadeTrait>,
}

impl PeerCatalogEntities {
    pub(crate) fn new(
        cache: Arc<dyn CatalogAgentCacheTrait>,
        mates_facade: Arc<dyn MatesFacadeTrait>,
    ) -> Self {
        PeerCatalogEntities {
            cache,
            mates_facade,
        }
    }
}

#[async_trait::async_trait]
impl PeerCatalogTrait for PeerCatalogEntities {
    async fn get_all_peer_catalogs(&self) -> Outcome<Vec<(Mates, Catalog)>> {
        let mates = self.mates_facade.get_all_mates().await?;
        let peer_catalog_cache = self.cache.get_peer_catalog_cache();

        let mut result = Vec::new();
        for mate in mates {
            match peer_catalog_cache.get_catalog(&mate.participant_id).await {
                Ok(Some(catalog)) => {
                    result.push((mate, catalog));
                }
                Ok(None) => {
                    // No catalog cached yet for this peer — skip silently.
                }
                Err(e) => {
                    warn!(
                        peer = %mate.participant_id,
                        error = %e,
                        "Failed to fetch peer catalog from cache; skipping peer"
                    );
                }
            }
        }

        Ok(result)
    }

    async fn get_peer_catalog(&self, peer_id: &String) -> Outcome<Option<Catalog>> {
        self.cache
            .get_peer_catalog_cache()
            .get_catalog(peer_id)
            .await
    }

    async fn set_peer_catalog(&self, peer_id: &String, catalog: &Catalog) -> Outcome<()> {
        self.cache
            .get_peer_catalog_cache()
            .set_catalog(peer_id, catalog)
            .await
    }
}
