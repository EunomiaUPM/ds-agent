pub(crate) mod peer_catalogs;

use ymir::errors::Outcome;
use crate::protocols::dsp::types::catalog_definition::Catalog;

#[async_trait::async_trait]
pub trait PeerCatalogTrait: Send + Sync {
    async fn get_peer_catalog(&self, peer_id: &String) -> Outcome<Option<Catalog>>;
    async fn set_peer_catalog(&self, peer_id: &String, catalog: &Catalog) -> Outcome<()>;
}
