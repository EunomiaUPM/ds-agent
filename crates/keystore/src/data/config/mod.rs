use crate::data::repo::config::KeystoreConfigRepo;
use common::config::ApplicationConfig;
use common::config::services::{
    CatalogConfig, CommonConfig, ContractsConfig, GatewayConfig, MonolithConfig, TransferConfig,
};
use std::ops::Deref;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ConfigPassthroughRepo {
    config: Arc<ApplicationConfig>,
}

impl KeystoreConfigRepo for ConfigPassthroughRepo {
    async fn get_transfer_config(&self) -> Outcome<TransferConfig> {
        Ok(self.config.transfer().clone())
    }

    async fn get_contracts_config(&self) -> Outcome<ContractsConfig> {
        Ok(self.config.contracts().clone())
    }

    async fn get_catalog_config(&self) -> Outcome<CatalogConfig> {
        Ok(self.config.catalog().clone())
    }

    async fn get_mono_config(&self) -> Outcome<MonolithConfig> {
        Ok(self.config.monolith().clone())
    }

    async fn get_gateway_config(&self) -> Outcome<GatewayConfig> {
        Ok(self.config.gateway().clone())
    }
    async fn get_config(&self) -> Outcome<ApplicationConfig> {
        Ok(self.config.deref().clone())
    }
}
