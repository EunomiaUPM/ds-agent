use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TransferDSPMessageType {
    TransferRequestMessage,
    TransferStartMessage,
    TransferCompletionMessage,
    TransferSuspensionMessage,
    TransferTerminationMessage,
    TransferProcess,
    TransferError,
}

impl Display for TransferDSPMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TransferDSPMessageType::TransferRequestMessage => "TransferRequestMessage".to_string(),
            TransferDSPMessageType::TransferStartMessage => "TransferStartMessage".to_string(),
            TransferDSPMessageType::TransferCompletionMessage => {
                "TransferCompletionMessage".to_string()
            }
            TransferDSPMessageType::TransferSuspensionMessage => {
                "TransferSuspensionMessage".to_string()
            }
            TransferDSPMessageType::TransferTerminationMessage => {
                "TransferTerminationMessage".to_string()
            }
            TransferDSPMessageType::TransferProcess => "TransferProcess".to_string(),
            TransferDSPMessageType::TransferError => "TransferError".to_string(),
        };
        write!(f, "{}", str)
    }
}
