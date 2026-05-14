use serde::Deserialize;
use crate::entities::role::Role;

/// Body for `POST /users` — admin only.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserCommand {
    pub tenant_id: String,
    pub email: String,
    pub password: String,
    pub role: Role,
    #[serde(default)]
    pub extra_fields: serde_json::Value,
}

/// Body for `PATCH /users/{tenant_id}`.
/// All fields are optional; omitted fields are left unchanged.
/// `role` is ignored for non-admin callers (enforced by the HTTP layer).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchUserCommand {
    pub email: Option<String>,
    pub role: Option<Role>,
    pub extra_fields: Option<serde_json::Value>,
}