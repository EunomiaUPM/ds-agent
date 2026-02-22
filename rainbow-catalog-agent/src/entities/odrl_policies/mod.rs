pub(crate) mod odrl_policies;

use crate::data::entities::odrl_offer;
use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use crate::entities::policy_templates::types::ParameterDefinition;
use rainbow_common::dsp_common::odrl::OdrlPolicyInfo;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OdrlPolicyDto {
    #[serde(flatten)]
    pub inner: odrl_offer::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CatalogEntityTypes {
    Distribution,
    DataService,
    Catalog,
    Dataset,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewOdrlPolicyDto {
    pub id: Option<Urn>,
    pub odrl_offer: OdrlPolicyInfo,
    pub entity_id: Urn,
    pub entity_type: CatalogEntityTypes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instantiation_parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Display for CatalogEntityTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CatalogEntityTypes::Distribution => "Distribution",
            CatalogEntityTypes::DataService => "DataService",
            CatalogEntityTypes::Catalog => "Catalog",
            CatalogEntityTypes::Dataset => "Dataset",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

impl From<NewOdrlPolicyDto> for NewOdrlOfferModel {
    fn from(dto: NewOdrlPolicyDto) -> Self {
        Self {
            id: dto.id,
            odrl_offer: dto.odrl_offer,
            entity_id: dto.entity_id,
            entity_type: dto.entity_type,
            source_template_id: dto.source_template_id,
            source_template_version: dto.source_template_version,
            instantiation_parameters: dto.instantiation_parameters,
            description: dto.description,
        }
    }
}

impl From<odrl_offer::Model> for OdrlPolicyDto {
    fn from(value: odrl_offer::Model) -> Self {
        Self { inner: value }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait OdrlPolicyEntityTrait: Sync + Send {
    async fn get_all_odrl_offers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<OdrlPolicyDto>>;
    async fn get_batch_odrl_offers(&self, ids: &Vec<Urn>) -> anyhow::Result<Vec<OdrlPolicyDto>>;
    async fn get_all_odrl_offers_by_entity(
        &self,
        entity: &Urn,
    ) -> anyhow::Result<Vec<OdrlPolicyDto>>;
    async fn get_odrl_offer_by_id(
        &self,
        odrl_offer_id: &Urn,
    ) -> anyhow::Result<Option<OdrlPolicyDto>>;
    async fn create_odrl_offer(
        &self,
        new_odrl_offer_model: &NewOdrlPolicyDto,
    ) -> anyhow::Result<OdrlPolicyDto>;
    async fn delete_odrl_offer_by_id(&self, odrl_offer_id: &Urn) -> anyhow::Result<()>;
    async fn delete_odrl_offers_by_entity(&self, entity_id: &Urn) -> anyhow::Result<()>;
}
