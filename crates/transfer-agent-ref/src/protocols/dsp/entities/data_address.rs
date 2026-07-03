use serde::{Deserialize, Serialize};

/// Pure DataAddress as defined in DSP
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataAddressDto {
    pub endpoint_type: String,
    pub endpoint: Option<String>,
    pub endpoint_properties: Option<Vec<EndpointPropertyDto>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointPropertyDto {
    pub name: String,
    pub value: String,
}
