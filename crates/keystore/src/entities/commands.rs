use crate::entities::key::Key;
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewParameterCommand<T> {
    pub key: Key,
    pub value: T,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditParameterCommand<T> {
    pub value: T,
    pub expected_version: Version,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewSecretCommand {
    pub key: Key,
    pub value: SecretValue,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSecretCommand {
    pub value: SecretValue,
    pub expected_version: Version,
    pub description: Option<String>,
}
