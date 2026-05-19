use crate::entities::key::Key;
use crate::entities::version::Version;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum RegistryError {
    #[error("entry not found: {0}")]
    NotFound(Key),

    #[error("entry already exists: {0}")]
    AlreadyExists(Key),

    #[error("version conflict on {key}: expected {expected:?}, actual {actual:?}")]
    VersionConflict {
        key: Key,
        expected: Version,
        actual: Version,
    },

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("deserialization failed for key {key}: {source}")]
    Deserialize {
        key: Key,
        #[source]
        source: serde_json::Error,
    },

    #[error("unauthorized")]
    Unauthorized,

    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
}
