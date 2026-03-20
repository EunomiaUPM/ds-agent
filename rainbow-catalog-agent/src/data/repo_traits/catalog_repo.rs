use crate::data::entities::catalog;
use crate::data::entities::catalog::{EditCatalogModel, NewCatalogModel};
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait CatalogRepositoryTrait: Send + Sync {
    async fn get_all_catalogs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
        with_main_catalog: bool,
    ) -> Outcome<Vec<catalog::Model>>;
    async fn get_batch_catalogs(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<catalog::Model>>;
    async fn get_catalog_by_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Option<catalog::Model>>;
    async fn get_main_catalog(
        &self,
    ) -> Outcome<Option<catalog::Model>>;

    async fn put_catalog_by_id(
        &self,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogModel,
    ) -> Outcome<catalog::Model>;
    async fn create_catalog(
        &self,
        new_catalog_model: &NewCatalogModel,
    ) -> Outcome<catalog::Model>;

    async fn create_main_catalog(
        &self,
        new_catalog_model: &NewCatalogModel,
    ) -> Outcome<catalog::Model>;

    async fn delete_catalog_by_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<()>;
}
