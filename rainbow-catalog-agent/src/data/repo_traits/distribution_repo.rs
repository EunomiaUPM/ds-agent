use crate::data::entities::distribution;
use crate::data::entities::distribution::{EditDistributionModel, NewDistributionModel};
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use rainbow_common::dcat_formats::DctFormats;
use urn::Urn;

#[async_trait::async_trait]
pub trait DistributionRepositoryTrait: Send + Sync {
    async fn get_all_distributions(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<distribution::Model>, CatalogAgentRepoErrors>;
    async fn get_batch_distributions(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<distribution::Model>, CatalogAgentRepoErrors>;

    async fn get_distributions_by_dataset_id(
        &self,
        dataset_id: &Urn,
    ) -> anyhow::Result<Vec<distribution::Model>, CatalogAgentRepoErrors>;
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        dataset_id: &Urn,
        dct_formats: &String,
    ) -> anyhow::Result<distribution::Model, CatalogAgentRepoErrors>;
    async fn get_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> anyhow::Result<Option<distribution::Model>, CatalogAgentRepoErrors>;
    async fn put_distribution_by_id(
        &self,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionModel,
    ) -> anyhow::Result<distribution::Model, CatalogAgentRepoErrors>;
    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionModel,
    ) -> anyhow::Result<distribution::Model, CatalogAgentRepoErrors>;
    async fn delete_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> anyhow::Result<(), CatalogAgentRepoErrors>;
}
