use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ymir::errors::Errors;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferState {
    #[serde(rename = "REQUESTED")]
    REQUESTED,
    #[serde(rename = "STARTED")]
    STARTED,
    #[serde(rename = "TERMINATED")]
    TERMINATED,
    #[serde(rename = "COMPLETED")]
    COMPLETED,
    #[serde(rename = "SUSPENDED")]
    SUSPENDED,
}

impl FromStr for TransferState {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REQUESTED" => Ok(Self::REQUESTED),
            "STARTED" => Ok(Self::STARTED),
            "TERMINATED" => Ok(Self::TERMINATED),
            "COMPLETED" => Ok(Self::COMPLETED),
            "SUSPENDED" => Ok(Self::SUSPENDED),
            _ => Err(Errors::parse("State not recognized", None)),
        }
    }
}

impl fmt::Display for TransferState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferState::REQUESTED => f.write_str("REQUESTED"),
            TransferState::STARTED => f.write_str("STARTED"),
            TransferState::TERMINATED => f.write_str("TERMINATED"),
            TransferState::COMPLETED => f.write_str("COMPLETED"),
            TransferState::SUSPENDED => f.write_str("SUSPENDED"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferStateAttribute {
    #[serde(rename = "ON_REQUEST")]
    OnRequest,
    #[serde(rename = "BY_PROVIDER")]
    ByProvider,
    #[serde(rename = "BY_CONSUMER")]
    ByConsumer,
}

impl FromStr for TransferStateAttribute {
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

impl fmt::Display for TransferStateAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferStateAttribute::OnRequest => f.write_str("ON_REQUEST"),
            TransferStateAttribute::ByProvider => f.write_str("BY_PROVIDER"),
            TransferStateAttribute::ByConsumer => f.write_str("BY_CONSUMER"),
        }
    }
}
