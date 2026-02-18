pub(crate) mod distributions;

use crate::data::entities::distribution;
use crate::data::entities::distribution::{EditDistributionModel, Model, NewDistributionModel};
use rainbow_common::dcat_formats::DctFormats;
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistributionDto {
    #[serde(flatten)]
    pub inner: distribution::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDistributionDto {
    pub id: Option<Urn>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dct_formats: Option<String>,
    pub dcat_access_service: String,
    pub dataset_id: Urn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDistributionDto {
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dcat_access_service: Option<String>,
}

impl From<NewDistributionDto> for NewDistributionModel {
    fn from(dto: NewDistributionDto) -> Self {
        Self {
            id: dto.id,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            dct_formats: dto.dct_formats,
            dcat_access_service: dto.dcat_access_service,
            dataset_id: dto.dataset_id,
        }
    }
}

impl From<EditDistributionDto> for EditDistributionModel {
    fn from(dto: EditDistributionDto) -> Self {
        Self {
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            dcat_access_service: dto.dcat_access_service,
        }
    }
}

impl From<distribution::Model> for DistributionDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait DistributionEntityTrait: Send + Sync {
    async fn get_all_distributions(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<DistributionDto>>;
    async fn get_batch_distributions(&self, ids: &Vec<Urn>)
        -> anyhow::Result<Vec<DistributionDto>>;

    async fn get_distributions_by_dataset_id(
        &self,
        dataset_id: &Urn,
    ) -> anyhow::Result<Vec<DistributionDto>>;
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        dataset_id: &Urn,
        dct_formats: &String,
    ) -> anyhow::Result<DistributionDto>;
    async fn get_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> anyhow::Result<Option<DistributionDto>>;
    async fn put_distribution_by_id(
        &self,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionDto,
    ) -> anyhow::Result<DistributionDto>;
    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionDto,
    ) -> anyhow::Result<DistributionDto>;
    async fn delete_distribution_by_id(&self, distribution_id: &Urn) -> anyhow::Result<()>;
}
