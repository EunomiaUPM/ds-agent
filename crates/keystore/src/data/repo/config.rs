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

use common::config::ApplicationConfig;
use common::config::services::{
    CatalogConfig, ContractsConfig, GatewayConfig, MonolithConfig, TransferConfig,
};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait KeystoreConfigRepo: Send + Sync {
    async fn get_transfer_config(&self) -> Outcome<TransferConfig>;
    async fn get_contracts_config(&self) -> Outcome<ContractsConfig>;
    async fn get_catalog_config(&self) -> Outcome<CatalogConfig>;
    async fn get_mono_config(&self) -> Outcome<MonolithConfig>;
    async fn get_gateway_config(&self) -> Outcome<GatewayConfig>;
    async fn get_config(&self) -> Outcome<ApplicationConfig>;
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum KeystoreConfigRepoErrors {
    #[error("Error fetching Keystore config. {0}")]
    ErrorFetching(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for KeystoreConfigRepoErrors {}
