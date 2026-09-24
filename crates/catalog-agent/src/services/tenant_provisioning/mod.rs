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

//! Bringing a tenant to a usable state: its main catalog and the main data service behind it.

pub mod listener;
pub mod service;

use crate::entities::catalogs::CatalogDto;
use crate::entities::data_services::DataServiceDto;
use common::auth::AccessScope;
use serde::Serialize;
use ymir::errors::Outcome;

#[derive(Debug, Clone, Serialize)]
pub struct ProvisionedTenantDto {
    pub tenant_id: String,
    pub catalog: CatalogDto,
    pub data_service: DataServiceDto,
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait TenantProvisioningServiceTrait: Send + Sync {
    /// Idempotent: existing main entities are kept, missing ones are created.
    async fn provision(
        &self,
        scope: &AccessScope,
        tenant_id: &str,
    ) -> Outcome<ProvisionedTenantDto>;
}
