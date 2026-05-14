use serde::{Deserialize, Serialize};
use crate::entities::role::Role;

/// Lightweight claims embedded in access tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: Role,
    pub iat: i64,
    pub exp: i64,
}

/// Claims embedded in refresh tokens.
/// `jti` ties this JWT to the revocation record in the DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub role: Role,
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
}

/// OIDC ID token claims (Core 1.0 §2 + custom `role` + caller-defined extras).
/// Carries full user identity so the relying party needs no extra round-trip.
/// Extra fields from `User::extra_fields` are flattened at the top level, so
/// `{ "department": "eng" }` becomes a first-class JWT claim, not a nested object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    /// Issuer — must match `OAuthConfig::issuer`.
    pub iss: String,
    /// Subject — tenant_id.
    pub sub: String,
    /// Audience — must match `OAuthConfig::audience`.
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub email: String,
    pub role: Role,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}