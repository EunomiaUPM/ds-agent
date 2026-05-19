use serde::Serialize;
use crate::entities::metadata::Metadata;
use crate::entities::secret_value::SecretValue;

#[derive(Clone, Debug, Serialize)]
pub struct Entry<T> {
    pub metadata: Metadata,
    pub value: T,
}

pub type SecretEntry = Entry<SecretValue>;