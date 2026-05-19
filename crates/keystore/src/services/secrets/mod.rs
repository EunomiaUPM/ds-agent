use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::metadata::Metadata;
use crate::entities::version::Version;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait SecretStore: Send + Sync {
    async fn create(&self, cmd: &NewSecretCommand, actor: &str) -> Outcome<Version>;

    async fn read(&self, key: &Key, actor: &str) -> Outcome<SecretEntry>;

    async fn read_metadata(&self, key: &Key, actor: &str) -> Outcome<Metadata>;

    async fn update(&self, key: &Key, cmd: &EditSecretCommand, actor: &str) -> Outcome<Version>;

    async fn delete(&self, key: &Key, actor: &str) -> Outcome<()>;

    async fn list(&self, prefix: &KeyPrefix, actor: &str) -> Outcome<Vec<Metadata>>;
}
