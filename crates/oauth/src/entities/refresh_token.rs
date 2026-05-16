use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: uuid::Uuid,
    pub tenant_id: String,
    /// JWT ID (`jti` claim) stored for revocation checks.
    pub jti: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
}
