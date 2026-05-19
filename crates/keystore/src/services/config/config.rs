use crate::data::repo::config::KeystoreConfigRepo;
use crate::services::config::ConfigStore;
use common::config::ApplicationConfig;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ConfigStoreImpl {
    repo: Arc<dyn KeystoreConfigRepo>,
}

impl ConfigStoreImpl {
    pub fn new(repo: Arc<dyn KeystoreConfigRepo>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl ConfigStore for ConfigStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_application_config(&self) -> Outcome<ApplicationConfig> {
        self.repo.get_config().await
    }
}
