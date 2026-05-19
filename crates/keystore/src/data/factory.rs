use crate::data::repo::config::KeystoreConfigRepo;
use crate::data::repo::parameters::ParameterRepoTrait;
use crate::data::repo::secrets::SecretRepoTrait;
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) trait DataFactory: Send + Sync {
    fn keystore_config_repo(&self) -> Arc<dyn KeystoreConfigRepo>;
    fn keystore_secrets_repo(&self) -> Arc<dyn SecretRepoTrait>;
    fn keystore_parameters_repo(&self) -> Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>;
}
