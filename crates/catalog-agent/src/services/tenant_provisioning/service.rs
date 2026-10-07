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

//! Provisioning over the catalog and data service services.

use std::str::FromStr;
use std::sync::Arc;

use common::oauth::{RoleTrait, UserInfo};
use common::facades::mates_facade::MatesFacadeTrait;
use urn::Urn;
use ymir::errors::Outcome;

use crate::entities::catalogs::NewCatalogDto;
use crate::entities::data_services::NewDataServiceDto;
use crate::services::catalogs::CatalogServiceTrait;
use crate::services::data_services::DataServiceServiceTrait;
use crate::services::tenant_provisioning::{ProvisionedTenantDto, TenantProvisioningServiceTrait};

pub struct TenantProvisioningService {
    catalogs: Arc<dyn CatalogServiceTrait>,
    data_services: Arc<dyn DataServiceServiceTrait>,
    mates: Arc<dyn MatesFacadeTrait>,
    /// DSP endpoint of this connector; the main data service points to it.
    dsp_url: String,
}

impl TenantProvisioningService {
    pub fn new(
        catalogs: Arc<dyn CatalogServiceTrait>,
        data_services: Arc<dyn DataServiceServiceTrait>,
        mates: Arc<dyn MatesFacadeTrait>,
        dsp_url: String,
    ) -> Self {
        Self {
            catalogs,
            data_services,
            mates,
            dsp_url,
        }
    }
}

#[async_trait::async_trait]
impl TenantProvisioningServiceTrait for TenantProvisioningService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn provision(&self, user: &UserInfo) -> Outcome<ProvisionedTenantDto> {
        user.require_root()?;

        let catalog = match self.catalogs.get_main_catalog(user).await? {
            Some(catalog) => catalog,
            None => {
                // The catalog publishes this connector's identity.
                let me = self.mates.get_me_mate().await?;
                let new_catalog = NewCatalogDto {
                    dspace_participant_id: Some(me.participant_id),
                    ..NewCatalogDto::default()
                };
                self.catalogs
                    .create_main_catalog(user, &new_catalog)
                    .await?
            }
        };

        let data_service = match self.data_services.get_main_data_service(user).await? {
            Some(data_service) => data_service,
            None => {
                let new_data_service = NewDataServiceDto {
                    dcat_endpoint_url: self.dsp_url.clone(),
                    catalog_id: Urn::from_str(&catalog.inner.id)?,
                    ..NewDataServiceDto::default()
                };
                self.data_services
                    .create_main_data_service(user, &new_data_service)
                    .await?
            }
        };

        Ok(ProvisionedTenantDto {
            catalog,
            data_service,
        })
    }
}
