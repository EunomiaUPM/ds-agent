use crate::entities::role::Role;
use chrono::{DateTime, Utc};

/// One user per tenant. `tenant_id` IS the primary key.
#[derive(Debug, Clone)]
pub struct User {
    pub tenant_id: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub extra_fields: serde_json::Value,
}
