/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Boot seeders of the catalog: the admin tenant's catalog and the policy template library.
//! Both act on the catalog services in-process, as the service token would through the API.

use std::sync::Arc;

use common::auth::AccessScope;
use common::boot::seeders::{BootPhase, BootSeeder};
use serde_json::Value;
use tokio::fs;
use ymir::errors::{Errors, Outcome};

use crate::entities::policy_templates::NewPolicyTemplateDto;
use crate::services::policy_templates::PolicyTemplateServiceTrait;
use crate::services::tenant_provisioning::TenantProvisioningServiceTrait;

/// Provisions the admin tenant with the same idempotent use case as any new tenant.
pub struct AdminTenantProvisioner {
    service: Arc<dyn TenantProvisioningServiceTrait>,
    tenant: String,
}

impl AdminTenantProvisioner {
    pub fn new(service: Arc<dyn TenantProvisioningServiceTrait>, tenant: String) -> Self {
        Self { service, tenant }
    }
}

#[async_trait::async_trait]
impl BootSeeder for AdminTenantProvisioner {
    fn name(&self) -> &'static str {
        "admin-tenant-provisioning"
    }

    /// In-process, so the tenant is ready before the first request is served.
    fn phase(&self) -> BootPhase {
        BootPhase::BeforeServe
    }

    async fn seed(&self) -> Outcome<()> {
        let provisioned = self
            .service
            .provision(&AccessScope::service(&self.tenant), &self.tenant)
            .await?;
        tracing::info!(
            catalog = provisioned.catalog.inner.id,
            data_service = provisioned.data_service.inner.id,
            tenant = self.tenant,
            "Admin tenant provisioned"
        );
        Ok(())
    }
}

/// Registers every `*.json` template of a folder; unreadable, invalid or already
/// registered templates are skipped, keeping boot idempotent.
pub struct PolicyTemplateLoader {
    service: Arc<dyn PolicyTemplateServiceTrait>,
    tenant: String,
    folder: String,
}

impl PolicyTemplateLoader {
    pub fn new(
        service: Arc<dyn PolicyTemplateServiceTrait>,
        tenant: String,
        folder: String,
    ) -> Self {
        Self {
            service,
            tenant,
            folder,
        }
    }

    async fn read_template(path: &std::path::Path) -> Option<NewPolicyTemplateDto> {
        let content = fs::read_to_string(path)
            .await
            .inspect_err(|e| tracing::error!("Failed to read file {path:?}: {e}"))
            .ok()?;
        let value: Value = serde_json::from_str(&content)
            .inspect_err(|e| tracing::error!("Invalid JSON format in file {path:?}: {e}"))
            .ok()?;
        serde_json::from_value(value)
            .inspect_err(|e| tracing::warn!("Policy template {path:?} rejected: {e}"))
            .ok()
    }

    /// A duplicate key means the template is already registered.
    fn is_duplicate(err: &Errors) -> bool {
        matches!(err, Errors::DatabaseError { reason, .. } if reason.contains("duplicate key"))
    }
}

#[async_trait::async_trait]
impl BootSeeder for PolicyTemplateLoader {
    fn name(&self) -> &'static str {
        "policy-templates"
    }

    fn phase(&self) -> BootPhase {
        BootPhase::BeforeServe
    }

    async fn seed(&self) -> Outcome<()> {
        let mut entries = match fs::read_dir(&self.folder).await {
            Ok(entries) => entries,
            Err(e) => {
                tracing::error!(
                    "Failed to read policy templates folder {}: {e}",
                    self.folder
                );
                return Ok(());
            }
        };
        let scope = AccessScope::service(&self.tenant);
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Some(template) = Self::read_template(&path).await else {
                continue;
            };
            match self.service.create_policy_template(&scope, &template).await {
                Ok(_) => {}
                Err(e) if Self::is_duplicate(&e) => tracing::info!(
                    "Policy template '{}' v{} already exists, skipping",
                    template.id.as_deref().unwrap_or("unknown"),
                    template.version.as_deref().unwrap_or("unknown")
                ),
                Err(e) => tracing::warn!("Policy template {path:?} rejected: {e}"),
            }
        }
        Ok(())
    }
}
