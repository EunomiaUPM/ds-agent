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

use std::sync::Arc;

use common::auth::ServiceHttpClient;
use common::boot::seeders::BootSeeder;
use serde_json::Value;
use tokio::fs;
use ymir::errors::{Errors, Outcome};

/// Provisions the admin tenant through the same idempotent endpoint as any new tenant.
pub struct AdminTenantProvisioner {
    client: Arc<ServiceHttpClient>,
    api_url: String,
    tenant: String,
}

impl AdminTenantProvisioner {
    /// `api_url` is the catalog management base, e.g. `http://host/api/v1/catalog-agent`.
    pub fn new(client: Arc<ServiceHttpClient>, api_url: String, tenant: String) -> Self {
        Self {
            client,
            api_url,
            tenant,
        }
    }

    fn string_at(value: &Value, pointer: &str) -> Outcome<String> {
        value
            .pointer(pointer)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| Errors::parse(format!("provisioning response lacks {pointer}"), None))
    }
}

#[async_trait::async_trait]
impl BootSeeder for AdminTenantProvisioner {
    fn name(&self) -> &'static str {
        "admin-tenant-provisioning"
    }

    async fn seed(&self) -> Outcome<()> {
        let url = format!("{}/tenants/{}/provision", self.api_url, self.tenant);
        let provisioned: Value = self
            .client
            .post_json(&url, Some(&self.tenant), &serde_json::json!({}))
            .await?;
        let catalog = Self::string_at(&provisioned, "/catalog/id")?;
        let data_service = Self::string_at(&provisioned, "/dataService/id")?;
        tracing::info!(
            catalog,
            data_service,
            tenant = self.tenant,
            "Admin tenant provisioned"
        );
        Ok(())
    }
}

/// Registers every `*.json` template of a folder; unreadable or rejected files are skipped.
pub struct PolicyTemplateLoader {
    client: Arc<ServiceHttpClient>,
    api_url: String,
    tenant: String,
    folder: String,
}

impl PolicyTemplateLoader {
    pub fn new(
        client: Arc<ServiceHttpClient>,
        api_url: String,
        tenant: String,
        folder: String,
    ) -> Self {
        Self {
            client,
            api_url,
            tenant,
            folder,
        }
    }

    async fn read_template(path: &std::path::Path) -> Option<Value> {
        let content = fs::read_to_string(path)
            .await
            .inspect_err(|e| tracing::error!("Failed to read file {path:?}: {e}"))
            .ok()?;
        serde_json::from_str(&content)
            .inspect_err(|e| tracing::error!("Invalid JSON format in file {path:?}: {e}"))
            .ok()
    }
}

#[async_trait::async_trait]
impl BootSeeder for PolicyTemplateLoader {
    fn name(&self) -> &'static str {
        "policy-templates"
    }

    async fn seed(&self) -> Outcome<()> {
        // Silent mode turns already-registered templates into no-ops, keeping boot idempotent.
        let url = format!("{}/policy-templates?silent=true", self.api_url);
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
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Some(template) = Self::read_template(&path).await else {
                continue;
            };
            if let Err(e) = self
                .client
                .post_json::<Value, Value>(&url, Some(&self.tenant), &template)
                .await
            {
                tracing::warn!("Policy template {path:?} rejected: {e}");
            }
        }
        Ok(())
    }
}
