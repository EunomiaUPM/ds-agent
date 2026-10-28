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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! The registry sync over the closures and the auth ports.

use std::sync::Arc;

use chrono::Utc;
use common::config::services::traits::CatalogConfigTrait;
use common::config::types::traits::MinKnownConfigTrait;
use common::oauth::OwnerScope;
use ymir::config::types::HostType;
use ymir::data::entities::shared::registered_dataset::Model as RegisteredDataset;
use ymir::errors::Outcome;

use crate::entities::resource_kind::ResourceKind;
use crate::services::registry::RegistryServiceTrait;
use crate::setup::context::AppContext;

pub struct RegistryService {
    ctx: Arc<AppContext>,
}

impl RegistryService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Where peers ask for the datasets: `<DSP base>/catalog/datasets`.
    fn location(&self) -> String {
        format!(
            "{}{}/datasets",
            self.ctx.config.contracts().get_host(HostType::Http),
            common::dsp_common::paths::CATALOG
        )
    }
}

#[async_trait::async_trait]
impl RegistryServiceTrait for RegistryService {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn sync(&self) -> Outcome<()> {
        let location = self.location();
        let (_, set) = self.ctx.closures().main_catalog(&OwnerScope::All).await?;
        let now = Utc::now();
        let datasets = set
            .iter()
            .filter(|resource| resource.kind() == ResourceKind::Dataset)
            .filter_map(|resource| {
                let owner = resource.owner();
                Some(RegisteredDataset {
                    id: resource.id().to_string(),
                    catalog_id: resource.catalog_id()?.to_string(),
                    location: location.clone(),
                    requirements: Vec::new(),
                    owner_user_id: owner.user_id.clone(),
                    owner_role: owner.role.clone(),
                    owner_visibility: owner.visibility.clone(),
                    updated_at: now,
                })
            })
            .collect();
        self.ctx.ports.auth.registry.sync_datasets(datasets).await
    }

    async fn resync(&self) {
        if let Err(error) = self.sync().await {
            tracing::warn!(%error, "datasets not registered with auth");
        }
    }
}
