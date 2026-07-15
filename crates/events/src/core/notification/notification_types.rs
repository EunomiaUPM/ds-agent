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

use crate::data::entities::notification;
use common::utils::get_urn_from_string;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[derive(Serialize, Deserialize)]
pub struct EventsNotificationResponse {
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

impl TryFrom<notification::Model> for EventsNotificationResponse {
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
pub enum EventsNotificationMessageTypes {
    RPCMessage,
    DSProtocolMessage,
    EntitiesMessage,
}

impl Display for EventsNotificationMessageTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventsNotificationMessageTypes::RPCMessage => Ok(f.write_str("RPCMessage")?),
            EventsNotificationMessageTypes::DSProtocolMessage => {
                Ok(f.write_str("DSProtocolMessage")?)
            }
            EventsNotificationMessageTypes::EntitiesMessage => {
                Ok(f.write_str("EunomiaDSAgentEntitiesMessage")?)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum EventsNotificationMessageCategory {
    TransferProcess,
    Catalog,
    ContractNegotiation,
    DataPlane,
}

impl Display for EventsNotificationMessageCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventsNotificationMessageCategory::TransferProcess => {
                Ok(f.write_str("TransferProcess")?)
            }
            EventsNotificationMessageCategory::Catalog => Ok(f.write_str("Catalog")?),
            EventsNotificationMessageCategory::ContractNegotiation => {
                Ok(f.write_str("ContractNegotiation")?)
            }
            EventsNotificationMessageCategory::DataPlane => Ok(f.write_str("DataPlane")?),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum EventsNotificationMessageOperation {
    Creation,
    Update,
    Deletion,
    IncomingMessage,
    OutgoingMessage,
}

impl Display for EventsNotificationMessageOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventsNotificationMessageOperation::Creation => Ok(f.write_str("Creation")?),
            EventsNotificationMessageOperation::Update => Ok(f.write_str("Update")?),
            EventsNotificationMessageOperation::Deletion => Ok(f.write_str("Deletion")?),
            EventsNotificationMessageOperation::IncomingMessage => {
                Ok(f.write_str("IncomingMessage")?)
            }
            EventsNotificationMessageOperation::OutgoingMessage => {
                Ok(f.write_str("OutgoingMessage")?)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum EventsNotificationStatus {
    Pending,
    Ok,
}

impl Display for EventsNotificationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventsNotificationStatus::Pending => Ok(f.write_str("Pending")?),
            EventsNotificationStatus::Ok => Ok(f.write_str("Ok")?),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EventsNotificationCreationRequest {
    pub category: EventsNotificationMessageCategory,
    pub subcategory: String,
    pub message_type: EventsNotificationMessageTypes,
    pub message_operation: EventsNotificationMessageOperation,
    pub message_content: serde_json::Value,
    pub status: EventsNotificationStatus,
}

#[derive(Serialize, Deserialize)]
pub struct EventsNotificationBroadcastRequest {
    pub category: EventsNotificationMessageCategory,
    pub subcategory: String,
    pub message_type: EventsNotificationMessageTypes,
    pub message_content: serde_json::Value,
    pub message_operation: EventsNotificationMessageOperation,
}
