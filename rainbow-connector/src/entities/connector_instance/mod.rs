pub mod connector_instance;
pub(crate) mod parameter_validator;
pub mod resolver;

use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::ConnectorMetadata;
use crate::entities::interaction::InteractionConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorInstantiationDto {
    pub template_name: String,
    pub template_version: String,
    pub distribution_id: Urn,
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<InstanceMetadataDto>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMetadataDto {
    pub description: Option<String>,
    pub owner_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorInstanceDto {
    pub id: Urn,
    #[serde(flatten)]
    pub metadata: ConnectorMetadata,
    pub authentication_config: AuthenticationConfig,
    pub interaction: InteractionConfig,
    pub distribution_id: Urn,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorInstanceTrait: Send + Sync {
    async fn get_instance_by_id(&self, id: &Urn) -> anyhow::Result<Option<ConnectorInstanceDto>>;
    async fn get_instance_by_distribution(
        &self,
        distribution_id: &Urn,
    ) -> anyhow::Result<Option<ConnectorInstanceDto>>;
    async fn upsert_instance(
        &self,
        instance_dto: &mut ConnectorInstantiationDto,
    ) -> anyhow::Result<ConnectorInstanceDto>;
    async fn delete_instance_by_id(&self, id: &Urn) -> anyhow::Result<()>;
}
