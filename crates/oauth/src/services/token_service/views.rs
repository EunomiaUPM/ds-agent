use serde::Serialize;

/// RFC 6749 §5.1 token response, extended with the OIDC `id_token`.
#[derive(Debug, Serialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub id_token: String,
}
