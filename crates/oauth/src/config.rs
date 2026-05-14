/// Runtime configuration for the OAuth / OIDC service.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// HS256 signing secret for access, refresh, and ID tokens.
    pub jwt_secret: String,
    /// Access token lifetime in seconds (default: 3 600 = 1 h).
    pub access_token_ttl_secs: i64,
    /// Refresh token lifetime in seconds (default: 2 592 000 = 30 d).
    pub refresh_token_ttl_secs: i64,
    /// OIDC `iss` claim and the base URL for the discovery document.
    /// Example: `"https://auth.example.com"`.
    pub issuer: String,
    /// OIDC `aud` claim — identifies the intended audience of ID tokens.
    /// Typically the client application identifier.
    pub audience: String,
}

impl OAuthConfig {
    pub fn new(jwt_secret: impl Into<String>, issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            access_token_ttl_secs: 3_600,
            refresh_token_ttl_secs: 2_592_000,
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

    pub fn with_access_ttl(mut self, secs: i64) -> Self {
        self.access_token_ttl_secs = secs;
        self
    }

    pub fn with_refresh_ttl(mut self, secs: i64) -> Self {
        self.refresh_token_ttl_secs = secs;
        self
    }
}
