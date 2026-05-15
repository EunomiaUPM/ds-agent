use serde::Serialize;

use crate::entities::role::Role;
use crate::entities::user::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserView {
    pub tenant_id: String,
    pub email: String,
    pub role: Role,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub extra_fields: serde_json::Value,
}

impl UserView {
    pub(crate) fn assemble(u: User) -> Self {
        Self {
            tenant_id: u.tenant_id,
            email: u.email,
            role: u.role,
            created_at: u.created_at,
            extra_fields: u.extra_fields,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserInfo {
    pub sub: String,
    pub email: String,
    pub role: Role,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl UserInfo {
    pub(crate) fn assemble(u: User) -> Self {
        let extra = match u.extra_fields {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };
        Self { sub: u.tenant_id, email: u.email, role: u.role, extra }
    }
}
