use crate::entities::key::Key;
use crate::entities::version::Version;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    pub key: Key,
    pub version: Version,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String, // TODO for multitenancy
    pub updated_by: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
}
