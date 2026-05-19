use crate::data::config::ConfigPassthroughRepo;
use crate::data::factory::DataFactory;
use crate::data::repo::config::KeystoreConfigRepo;
use crate::data::repo::parameters::ParameterRepoTrait;
use crate::data::repo::secrets::SecretRepoTrait;
use crate::data::sea_orm::repos::parameter::SeaOrmParameterRepo;
use crate::data::sea_orm::repos::secret::SeaOrmSecretRepo;
use common::config::ApplicationConfig;
use sea_orm::DatabaseConnection;
use std::ops::Deref;
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) struct SeaOrmFactory {
    config: Arc<ApplicationConfig>,
    db: Arc<DatabaseConnection>,
}

#[allow(dead_code)]
impl SeaOrmFactory {
    pub fn new(db: Arc<DatabaseConnection>, config: Arc<ApplicationConfig>) -> Self {
        Self { db, config }
    }
}

impl DataFactory for SeaOrmFactory {
    fn keystore_config_repo(&self) -> Arc<dyn KeystoreConfigRepo> {
        Arc::new(ConfigPassthroughRepo::new(self.config.clone()))
    }

    fn keystore_secrets_repo(&self) -> Arc<dyn SecretRepoTrait> {
        Arc::new(SeaOrmSecretRepo::new(self.db.deref().clone()))
    }

    fn keystore_parameters_repo(&self) -> Arc<dyn ParameterRepoTrait<Value = serde_json::Value>> {
        Arc::new(SeaOrmParameterRepo::new(self.db.deref().clone()))
    }
}
