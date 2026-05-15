use serde::{Deserialize, Serialize};

use crate::entities::role::Role;

#[derive(Serialize, Deserialize)]
pub(crate) struct AccessClaims {
    pub sub: String,
    pub role: Role,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RefreshClaims {
    pub sub: String,
    pub role: Role,
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub email: String,
    pub role: Role,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

pub(crate) fn as_map(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match v {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    }
}
