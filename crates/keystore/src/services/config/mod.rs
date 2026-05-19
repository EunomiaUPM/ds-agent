use common::config::ApplicationConfig;
use ymir::errors::Outcome;

pub(crate) mod config;

#[async_trait::async_trait]
pub trait ConfigStore: Send + Sync {
    async fn get_application_config(&self) -> Outcome<ApplicationConfig>;
}
