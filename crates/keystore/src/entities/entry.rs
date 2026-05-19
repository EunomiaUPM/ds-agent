use crate::entities::metadata::Metadata;
use crate::entities::secret_value::SecretValue;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Entry<T> {
    pub metadata: Metadata,
    pub value: T,
}

pub type SecretEntry = Entry<SecretValue>;
