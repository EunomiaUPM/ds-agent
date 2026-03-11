//! Connector instance DTOs and the instance service trait.
//!
//! A *connector instance* is a resolved, distribution-specific configuration
//! derived from a connector template by binding concrete parameter values.
//!
//! # Lifecycle
//!
//! 1. A caller submits a [`ConnectorInstantiationDto`] that names the template
//!    and supplies parameter values.
//! 2. [`ConnectorInstanceTrait::upsert_instance`] validates the parameters,
//!    enriches them with system defaults, resolves all `{{__PARAM__}}`
//!    placeholders, and persists the result.
//! 3. The resolved [`ConnectorInstanceDto`] is returned and stored in the
//!    database for the dataplane to query at transfer time.

pub mod service;

use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::ConnectorMetadata;
use crate::entities::interaction::InteractionConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;

/// Request payload for creating or updating a connector instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorInstantiationDto {
    /// Name of the connector template to instantiate.
    pub template_name: String,
    /// Version of the connector template.
    pub template_version: String,
    /// Distribution this instance belongs to.
    pub distribution_id: Urn,
    /// User-supplied parameter values.  Missing required parameters cause a
    /// validation error; parameters not declared in the template are silently
    /// ignored.
    pub parameters: HashMap<String, serde_json::Value>,
    /// Optional display metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<InstanceMetadataDto>,
    /// When `true` the instance is fully resolved and validated but **not**
    /// persisted.  Used for pre-flight checks.
    #[serde(default)]
    pub dry_run: bool,
}

/// Optional human-readable metadata attached to a connector instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMetadataDto {
    pub description: Option<String>,
    pub owner_id: Option<String>,
}

/// A fully resolved connector instance ready for use by the dataplane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorInstanceDto {
    pub id: Urn,
    #[serde(flatten)]
    pub metadata: ConnectorMetadata,
    /// Resolved authentication configuration (all placeholders substituted).
    pub authentication_config: AuthenticationConfig,
    /// Resolved interaction configuration (all placeholders substituted).
    pub interaction: InteractionConfig,
    pub distribution_id: Urn,
}

/// Service interface for connector instance operations.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorInstanceTrait: Send + Sync {
    async fn get_instance_by_id(&self, id: &Urn) -> anyhow::Result<Option<ConnectorInstanceDto>>;
    async fn get_instance_by_distribution(
        &self,
        distribution_id: &Urn,
    ) -> anyhow::Result<Option<ConnectorInstanceDto>>;
    /// Validate parameters, resolve placeholders, and persist the instance.
    ///
    /// Idempotent: if an instance already exists for `distribution_id` it is
    /// updated in-place.
    async fn upsert_instance(
        &self,
        instance_dto: &mut ConnectorInstantiationDto,
    ) -> anyhow::Result<ConnectorInstanceDto>;
    async fn delete_instance_by_id(&self, id: &Urn) -> anyhow::Result<()>;
}
