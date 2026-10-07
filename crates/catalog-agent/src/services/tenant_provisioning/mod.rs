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

//! Bringing the connector to a usable state: its main catalog and the main data service behind
//! it, one of each for the whole connector (the root's, public). Every user publishes in it
//! and creates the sub-catalogs it wants; each peer sees the ones it may.

// Provisioned one tenant per user created in the former OAuth module; with one main catalog
// per connector there is nothing to provision per user. Kept as it was, out of the tree.
// pub mod listener;
pub mod service;

use crate::entities::catalogs::CatalogDto;
use crate::entities::data_services::DataServiceDto;
use common::oauth::UserInfo;
use serde::Serialize;
use ymir::errors::Outcome;

/// Main catalog and data service of the connector.
#[derive(Debug, Clone, Serialize)]
pub struct ProvisionedTenantDto {
    pub catalog: CatalogDto,
    pub data_service: DataServiceDto,
}

/// Giving the connector its main catalog and data service.
#[mockall::automock]
#[async_trait::async_trait]
pub trait TenantProvisioningServiceTrait: Send + Sync {
    /// Idempotent: existing main entities are kept, missing ones are created. The root only.
    async fn provision(&self, user: &UserInfo) -> Outcome<ProvisionedTenantDto>;
}
