use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ymir::errors::Errors;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferDSPState {
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

impl FromStr for TransferDSPState {
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

impl fmt::Display for TransferDSPState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferDSPState::REQUESTED => f.write_str("REQUESTED"),
            TransferDSPState::STARTED => f.write_str("STARTED"),
            TransferDSPState::TERMINATED => f.write_str("TERMINATED"),
            TransferDSPState::COMPLETED => f.write_str("COMPLETED"),
            TransferDSPState::SUSPENDED => f.write_str("SUSPENDED"),
        }
    }
}
