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

use crate::protocols::dsp::types::catalog_definition::Catalog;
use crate::services::peer_catalogs::PeerCatalogServiceTrait;
use common::auth::AccessScope;
use common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use ymir::errors::Outcome;

pub struct OrchestrationPersistenceForProtocolForRPC {
    peer_catalog_entity_service: Arc<dyn PeerCatalogServiceTrait>,
}

impl OrchestrationPersistenceForProtocolForRPC {
    pub fn new(peer_catalog_entity_service: Arc<dyn PeerCatalogServiceTrait>) -> Self {
        Self {
            peer_catalog_entity_service,
        }
    }

    pub async fn get_catalog(
        &self,
        scope: &AccessScope,
        peer_id: &str,
    ) -> Outcome<Option<Catalog>> {
        let catalog = self
            .peer_catalog_entity_service
            .get_peer_catalog(scope, peer_id)
            .await?;
        Ok(catalog)
    }

    pub async fn set_catalog(
        &self,
        scope: &AccessScope,
        peer_id: &str,
        catalog: &Catalog,
    ) -> Outcome<()> {
        let _ = self
            .peer_catalog_entity_service
            .set_peer_catalog(scope, peer_id, catalog)
            .await?;
        Ok(())
    }
}
