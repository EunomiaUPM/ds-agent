use serde::{Deserialize, Serialize};

/// Where an API key is attached to the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiKeyLocation {
    Header,
    Query,
}
