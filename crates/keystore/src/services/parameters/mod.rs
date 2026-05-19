// services/parameter_store.rs
use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::metadata::Metadata;
use crate::entities::version::Version;
use serde::Serialize;
use serde::de::DeserializeOwned;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ParameterStore<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn create(&self, cmd: &NewParameterCommand<T>, actor: &str) -> Outcome<Version>;

    async fn read(&self, key: &Key, actor: &str) -> Outcome<Entry<T>>;

    async fn update(
        &self,
        key: &Key,
        cmd: &EditParameterCommand<T>,
        actor: &str,
    ) -> Outcome<Version>;

    async fn delete(&self, key: &Key, actor: &str) -> Outcome<()>;

    async fn list(&self, prefix: &KeyPrefix, actor: &str) -> Outcome<Vec<Metadata>>;
}
