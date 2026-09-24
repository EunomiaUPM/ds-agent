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

use std::str::FromStr;
use std::sync::Arc;

use common::auth::{AccessScope, RbacRole};
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::entities::catalogs::NewCatalogDto;
use crate::entities::data_services::NewDataServiceDto;
use crate::services::catalogs::CatalogServiceTrait;
use crate::services::data_services::DataServiceServiceTrait;
use crate::services::tenant_provisioning::{ProvisionedTenantDto, TenantProvisioningServiceTrait};

pub struct TenantProvisioningService {
    catalogs: Arc<dyn CatalogServiceTrait>,
    data_services: Arc<dyn DataServiceServiceTrait>,
    mates: Arc<dyn MatesFacadeTrait>,
    /// DSP endpoint of this connector; every tenant's main data service points to it.
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
    async fn provision(
        &self,
        scope: &AccessScope,
        tenant_id: &str,
    ) -> Outcome<ProvisionedTenantDto> {
        scope.require_write()?;
        if !scope.permits(tenant_id) {
            return Err(Errors::forbidden(
                "forbidden: cannot provision another tenant",
                None,
            ));
        }
        let owner = AccessScope::from_role(RbacRole::Owner, tenant_id);

        let catalog = match self.catalogs.get_main_catalog(&owner).await? {
            Some(catalog) => catalog,
            None => {
                // The catalog publishes this connector's identity, shared by all its tenants.
                let me = self.mates.get_me_mate(tenant_id.to_string()).await?;
                let new_catalog = NewCatalogDto {
                    dspace_participant_id: Some(me.participant_id),
                    ..NewCatalogDto::default()
                };
                self.catalogs
                    .create_main_catalog(&owner, &new_catalog)
                    .await?
            }
        };

        let data_service = match self.data_services.get_main_data_service(&owner).await? {
            Some(data_service) => data_service,
            None => {
                let new_data_service = NewDataServiceDto {
                    dcat_endpoint_url: self.dsp_url.clone(),
                    catalog_id: Urn::from_str(&catalog.inner.id)?,
                    ..NewDataServiceDto::default()
                };
                self.data_services
                    .create_main_data_service(&owner, &new_data_service)
                    .await?
            }
        };

        Ok(ProvisionedTenantDto {
            tenant_id: tenant_id.to_string(),
            catalog,
            data_service,
        })
    }
}
