use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::entities::role::Role;
use crate::entities::user::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    pub tenant_id: String,
    pub email: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub extra_fields: serde_json::Value,
}

impl From<User> for UserView {
    fn from(u: User) -> Self {
        Self { tenant_id: u.tenant_id, email: u.email, role: u.role, created_at: u.created_at, extra_fields: u.extra_fields }
    }
}