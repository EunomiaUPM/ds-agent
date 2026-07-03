use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ymir::errors::Errors;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferDSPStateAttribute {
    #[serde(rename = "ON_REQUEST")]
    OnRequest,
    #[serde(rename = "BY_PROVIDER")]
    ByProvider,
    #[serde(rename = "BY_CONSUMER")]
    ByConsumer,
}

impl FromStr for TransferDSPStateAttribute {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ON_REQUEST" => Ok(Self::OnRequest),
            "BY_PROVIDER" => Ok(Self::ByProvider),
            "BY_CONSUMER" => Ok(Self::ByConsumer),
            _ => Err(Errors::parse("State Attribute not recognized", None)),
        }
    }
}

impl fmt::Display for TransferDSPStateAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferDSPStateAttribute::OnRequest => f.write_str("ON_REQUEST"),
            TransferDSPStateAttribute::ByProvider => f.write_str("BY_PROVIDER"),
            TransferDSPStateAttribute::ByConsumer => f.write_str("BY_CONSUMER"),
        }
    }
}
