use common::config::ApplicationConfig;
use common::config::services::{
    CatalogConfig, CommonConfig, ContractsConfig, GatewayConfig, MonolithConfig, TransferConfig,
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
pub enum KeystoreConfigRepoErrors {
    #[error("Error fetching Keystore config. {0}")]
    ErrorFetching(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for KeystoreConfigRepoErrors {}
