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
