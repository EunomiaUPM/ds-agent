/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// DSP Message types

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
