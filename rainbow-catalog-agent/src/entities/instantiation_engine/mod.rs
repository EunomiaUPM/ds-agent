pub(crate) mod instantiation_engine;
pub(crate) mod validator_request;
pub(crate) mod validators;

use crate::entities::common::PolicyTemplateAllowedDefaultValues;
use crate::entities::odrl_policies::CatalogEntityTypes;
use crate::OdrlPolicyDto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyInstantiationDto {
    id: String,
    version: String,
    parameters: HashMap<String, PolicyTemplateAllowedDefaultValues>,
    entity_id: Urn,
    entity_type: CatalogEntityTypes,
    pub description: Option<String>,
}

#[async_trait::async_trait]
pub trait PolicyInstantiationTrait: Send + Sync {
    async fn instantiate_policy(
        &self,
        instantiation_request: &NewPolicyInstantiationDto,
    ) -> Outcome<OdrlPolicyDto>;
}
