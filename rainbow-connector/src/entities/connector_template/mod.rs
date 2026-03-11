//! Connector template DTOs and the template service trait.
//!
//! A *connector template* is the reusable, parameterised blueprint from which
//! connector instances are created.  It declares:
//!
//! - [`ConnectorMetadata`] — name, version, author, description.
//! - An [`AuthenticationConfig`] section with `{{__PARAM__}}` placeholders.
//! - An [`InteractionConfig`] section with `{{__PARAM__}}` placeholders.
//! - A `parameters` list ([`ParameterDefinition`]) that declares the name, type,
//!   and optional default for every placeholder used in the template.
//!
//! When a template is created the engine validates that every placeholder in
//! the auth/interaction sections has a matching declaration in `parameters`, and
//! vice-versa.
//!
//! [`AuthenticationConfig`]: crate::entities::auth_config::AuthenticationConfig
//! [`InteractionConfig`]: crate::entities::interaction::InteractionConfig
//! [`ParameterDefinition`]: crate::entities::parameters::ParameterDefinition

pub(crate) mod service;

use crate::data::entities::connector_templates::NewConnectorTemplateModel;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::interaction::InteractionConfig;
use crate::entities::parameters::ParameterDefinition;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Display and versioning metadata for a connector template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorMetadata {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
}

/// The full connector template: metadata + parameterised auth/interaction + declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorTemplateDto {
    #[serde(flatten)]
    pub metadata: ConnectorMetadata,
    pub authentication: AuthenticationConfig,
    pub interaction: InteractionConfig,
    /// Declared parameter definitions — every `{{__NAME__}}` placeholder used in
    /// the auth or interaction sections must have a matching entry here.
    pub parameters: Vec<ParameterDefinition>,
}

impl TryFrom<ConnectorTemplateDto> for NewConnectorTemplateModel {
    type Error = anyhow::Error;

    fn try_from(value: ConnectorTemplateDto) -> Result<Self, Self::Error> {
        let authentication = serde_json::to_value(value.authentication)?;
        let interaction = serde_json::to_value(value.interaction)?;
        let parameters = serde_json::to_value(value.parameters)?;
        Ok(Self {
            name: Option::from(value.metadata.name.clone()),
            version: Option::from(value.metadata.version.clone()),
            author: Option::from(value.metadata.author.clone()),
            spec: json!({
                "authentication": authentication,
                "interaction": interaction,
                "parameters": parameters,
            }),
        })
    }
}

/// Service interface for connector template CRUD operations.
#[async_trait::async_trait]
pub trait ConnectorTemplateEntitiesTrait: Send + Sync {
    async fn get_all_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<ConnectorTemplateDto>>;
    async fn get_templates_by_id(
        &self,
        template_id: &String,
    ) -> anyhow::Result<Vec<ConnectorTemplateDto>>;
    async fn get_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> anyhow::Result<Option<ConnectorTemplateDto>>;
    async fn create_template(
        &self,
        new_template: &mut ConnectorTemplateDto,
    ) -> anyhow::Result<ConnectorTemplateDto>;
    async fn delete_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> anyhow::Result<()>;
}
