pub(crate) mod data_services;

use crate::data::entities::dataservice;
use crate::data::entities::dataservice::{EditDataServiceModel, Model, NewDataServiceModel};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use urn::{Urn, UrnBuilder};
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataServiceDto {
    #[serde(flatten)]
    pub inner: dataservice::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDataServiceDto {
    pub id: Option<Urn>,
    pub dcat_endpoint_description: Option<String>,
    pub dcat_endpoint_url: String,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub catalog_id: Urn,
}

impl Default for NewDataServiceDto {
    fn default() -> Self {
        Self {
            id: None,
            dcat_endpoint_description: None,
            dcat_endpoint_url: "".to_string(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_title: None,
            dct_description: None,
            catalog_id: Urn::from_str("urn:fake-urn:000").unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDataServiceDto {
    pub dcat_endpoint_description: Option<String>,
    pub dcat_endpoint_url: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
}

impl From<NewDataServiceDto> for NewDataServiceModel {
    fn from(dto: NewDataServiceDto) -> Self {
        Self {
            id: dto.id,
            dcat_endpoint_description: dto.dcat_endpoint_description,
            dcat_endpoint_url: dto.dcat_endpoint_url,
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            catalog_id: dto.catalog_id,
            dspace_main_data_service: false,
        }
    }
}

impl From<EditDataServiceDto> for EditDataServiceModel {
    fn from(dto: EditDataServiceDto) -> Self {
        Self {
            dcat_endpoint_description: dto.dcat_endpoint_description,
            dcat_endpoint_url: dto.dcat_endpoint_url,
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
        }
    }
}

impl From<dataservice::Model> for DataServiceDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataServiceEntityTrait: Send + Sync {
    async fn get_all_data_services(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DataServiceDto>>;
    async fn get_batch_data_services(&self, ids: &Vec<Urn>) -> Outcome<Vec<DataServiceDto>>;

    async fn get_data_services_by_catalog_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DataServiceDto>>;

    async fn get_main_data_service(&self) -> Outcome<Option<DataServiceDto>>;
    async fn get_data_service_by_id(
        &self,
        data_service_id: &Urn,
    ) -> Outcome<Option<DataServiceDto>>;
    async fn put_data_service_by_id(
        &self,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn create_data_service(
        &self,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn create_main_data_service(
        &self,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn delete_data_service_by_id(&self, data_service_id: &Urn) -> Outcome<()>;
}
