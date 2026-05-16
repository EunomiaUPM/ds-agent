use crate::entities::role::Role;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateUserCommand {
    pub tenant_id: String,
    pub email: String,
    pub password: String,
    pub role: Role,
    #[serde(default)]
    pub extra_fields: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatchUserCommand {
    pub email: Option<String>,
    pub role: Option<Role>,
    pub extra_fields: Option<serde_json::Value>,
}
