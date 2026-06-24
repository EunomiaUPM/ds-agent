pub mod service;
pub mod views;

use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait SecretStore: Send + Sync {
    async fn create(&self, cmd: &NewSecretCommand) -> Outcome<SecretEntry>;

    async fn read(&self, key: &Key) -> Outcome<SecretEntry>;

    async fn update(&self, key: &Key, cmd: &EditSecretCommand) -> Outcome<Version>;

    async fn delete(&self, key: &Key) -> Outcome<()>;

    async fn list(&self, prefix: &KeyPrefix) -> Outcome<Vec<SecretEntry>>;

    async fn upsert(&self, key: &Key, value: SecretValue) -> Outcome<()>;
}
