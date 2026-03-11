use serde::{Deserialize, Serialize};

/// OAuth 2.0 grant type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthGrantType {
    ClientCredentials,
    AuthorizationCode,
}
