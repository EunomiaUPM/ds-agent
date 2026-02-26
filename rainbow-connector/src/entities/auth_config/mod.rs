pub(crate) mod parameter_validation;
pub(crate) mod resolver;

use crate::entities::common::parameters::{TemplateString, TemplateVecString};
use crate::entities::common::secret_management::SecretString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthenticationConfig {
    NoAuth,
    BasicAuth(BasicAuthConfig),
    BearerToken {
        token: SecretString,
    },
    ApiKey {
        key: TemplateString,
        value: SecretString,
        location: ApiKeyLocation, // Enum: Header, Query
    },
    OAuth2 {
        grant_type: OAuthGrantType, // Enum: ClientCredentials, AuthorizationCode...
        token_url: TemplateString,
        client_id: TemplateString,
        client_secret: SecretString,
        scopes: TemplateVecString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub username: TemplateString,
    pub password: SecretString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthGrantType {
    ClientCredentials,
    AuthorizationCode,
}
