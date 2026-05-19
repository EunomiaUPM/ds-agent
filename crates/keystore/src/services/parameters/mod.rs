pub mod service;
pub mod views;

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
    async fn create(&self, cmd: &NewParameterCommand<T>) -> Outcome<Version>;

    async fn read(&self, key: &Key) -> Outcome<Entry<T>>;

    async fn update(
        &self,
        key: &Key,
        cmd: &EditParameterCommand<T>,
        actor: &str,
    ) -> Outcome<Version>;

    async fn delete(&self, key: &Key) -> Outcome<()>;

    async fn list(&self, prefix: &KeyPrefix) -> Outcome<Vec<Metadata>>;
}
