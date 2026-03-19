use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthProtocolTypes {
    #[serde(rename = "OAuth")]
    OAuth,
    #[serde(rename = "Token")]
    OpaqueToken,
    #[serde(rename = "GNAP")]
    Gnap
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum DSPProtocolVersions {
    #[serde(rename = "2024-1")]
    V2024_1,
    #[serde(rename = "2025-1")]
    V2025_1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DSPIdentifierTypes {
    #[serde(rename = "did:web")]
    DidWeb,
    #[serde(rename = "did:jwk")]
    DidJWK,
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "D-U-N-S")]
    DUNS
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DSPBindings {
    #[serde(rename = "HTTPS")]
    HTTPS,
    #[serde(rename = "HTTP")]
    HTTP
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    pub protocol_versions: Vec<Version>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub binding: DSPBindings,
    pub path: String,
    pub version: DSPProtocolVersions,
    pub auth: Option<Auth>,
    pub identifier_type: Option<DSPIdentifierTypes>,
    pub service_id: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub protocol: AuthProtocolTypes,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionPath {
    pub path: String
}
