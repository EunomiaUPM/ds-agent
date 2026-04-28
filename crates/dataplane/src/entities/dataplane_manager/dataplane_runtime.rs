use connector::ApiKeyLocation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DataplaneRuntime {
    pub auth: ResolvedAuthCredentials,
    pub subscription: serde_json::Value,
    pub unsubscription: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolvedAuthCredentials {
    #[default]
    NoAuth,
    BearerToken {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    OAuth2 {
        access_token: String,
        token_type: String,
        /// Unix timestamp (seconds) at which the token expires, if known.
        expires_at: Option<u64>,
    },
}
