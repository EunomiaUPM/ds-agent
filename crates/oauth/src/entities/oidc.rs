use serde::Serialize;
use crate::entities::role::Role;

/// Response body for `GET /userinfo` (OIDC Core 1.0 §5.3).
/// Extra fields are flattened to the top level, matching the ID token layout.
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: String,
    pub role: Role,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// Token response (RFC 6749 §5.1 + OIDC Core §3.1.3.3) ───────────────────────

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    /// OIDC ID token — signed JWT carrying full user identity.
    pub id_token: String,
}
