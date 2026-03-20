/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::notification;
use common::utils::get_urn_from_string;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[derive(Serialize, Deserialize)]
pub struct RainbowEventsNotificationResponse {
    #[serde(rename = "notificationId")]
    pub id: Urn,
    pub timestamp: chrono::NaiveDateTime,
    pub category: String,
    pub subcategory: String,
    #[serde(rename = "messageType")]
    pub message_type: String,
    #[serde(rename = "messageOperation")]
    pub message_operation: String,
    #[serde(rename = "messageContent")]
    pub message_content: serde_json::Value,
    #[serde(rename = "subscriptionId")]
    pub subscription_id: Urn,
}

impl TryFrom<notification::Model> for RainbowEventsNotificationResponse {
    type Error = Errors;

    fn try_from(value: notification::Model) -> Outcome<Self> {
        Ok(Self {
            id: get_urn_from_string(&value.id)?,
            timestamp: value.timestamp,
            category: value.category,
            subcategory: value.subcategory,
            message_type: value.message_type,
            message_content: value.message_content,
            message_operation: value.message_operation,
            subscription_id: get_urn_from_string(&value.subscription_id)?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub enum RainbowEventsNotificationMessageTypes {
    RPCMessage,
    DSProtocolMessage,
    RainbowEntitiesMessage,
}

impl Display for RainbowEventsNotificationMessageTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RainbowEventsNotificationMessageTypes::RPCMessage => Ok(f.write_str("RPCMessage")?),
            RainbowEventsNotificationMessageTypes::DSProtocolMessage => {
                Ok(f.write_str("DSProtocolMessage")?)
            }
            RainbowEventsNotificationMessageTypes::RainbowEntitiesMessage => {
                Ok(f.write_str("RainbowEntitiesMessage")?)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum RainbowEventsNotificationMessageCategory {
    TransferProcess,
    Catalog,
    ContractNegotiation,
    DataPlane,
}

impl Display for RainbowEventsNotificationMessageCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RainbowEventsNotificationMessageCategory::TransferProcess => {
                Ok(f.write_str("TransferProcess")?)
            }
            RainbowEventsNotificationMessageCategory::Catalog => Ok(f.write_str("Catalog")?),
            RainbowEventsNotificationMessageCategory::ContractNegotiation => {
                Ok(f.write_str("ContractNegotiation")?)
            }
            RainbowEventsNotificationMessageCategory::DataPlane => Ok(f.write_str("DataPlane")?),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum RainbowEventsNotificationMessageOperation {
    Creation,
    Update,
    Deletion,
    IncomingMessage,
    OutgoingMessage,
}

impl Display for RainbowEventsNotificationMessageOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RainbowEventsNotificationMessageOperation::Creation => Ok(f.write_str("Creation")?),
            RainbowEventsNotificationMessageOperation::Update => Ok(f.write_str("Update")?),
            RainbowEventsNotificationMessageOperation::Deletion => Ok(f.write_str("Deletion")?),
            RainbowEventsNotificationMessageOperation::IncomingMessage => {
                Ok(f.write_str("IncomingMessage")?)
            }
            RainbowEventsNotificationMessageOperation::OutgoingMessage => {
                Ok(f.write_str("OutgoingMessage")?)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum RainbowEventsNotificationStatus {
    Pending,
    Ok,
}

impl Display for RainbowEventsNotificationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RainbowEventsNotificationStatus::Pending => Ok(f.write_str("Pending")?),
            RainbowEventsNotificationStatus::Ok => Ok(f.write_str("Ok")?),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RainbowEventsNotificationCreationRequest {
    pub category: RainbowEventsNotificationMessageCategory,
    pub subcategory: String,
    pub message_type: RainbowEventsNotificationMessageTypes,
    pub message_operation: RainbowEventsNotificationMessageOperation,
    pub message_content: serde_json::Value,
    pub status: RainbowEventsNotificationStatus,
}

#[derive(Serialize, Deserialize)]
pub struct RainbowEventsNotificationBroadcastRequest {
    pub category: RainbowEventsNotificationMessageCategory,
    pub subcategory: String,
    pub message_type: RainbowEventsNotificationMessageTypes,
    pub message_content: serde_json::Value,
    pub message_operation: RainbowEventsNotificationMessageOperation,
}
