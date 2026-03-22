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

use crate::data::entities::subscription;
use crate::data::repo::NewSubscription;
use common::utils::get_urn_from_string;
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[derive(PartialEq)]
pub enum EventsSubscriptionCreationTypes {
    TransferProcess(EventsSubscriptionCreationRequestForTransferProcess),
    Catalog(EventsSubscriptionCreationRequestForCatalog),
    ContractNegotiation(EventsSubscriptionCreationRequestForContractNegotiation),
    DataPlane(EventsSubscriptionCreationRequestForDataPlane),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EventsSubscriptionCreationRequest {
    #[serde(rename = "callbackAddress")]
    pub callback_address: String,
    #[serde(rename = "expirationTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventsSubscriptionCreationRequestForTransferProcess {
    #[serde(rename = "callbackAddress")]
    pub callback_address: String,
    #[serde(rename = "expirationTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

impl Into<NewSubscription> for EventsSubscriptionCreationRequestForTransferProcess {
    fn into(self) -> NewSubscription {
        NewSubscription {
            callback_address: self.callback_address,
            transfer_process: true,
            contract_negotiation_process: false,
            catalog: false,
            data_plane: false,
            active: true,
            expiration_time: self.expiration_time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventsSubscriptionCreationRequestForCatalog {
    #[serde(rename = "callbackAddress")]
    pub callback_address: String,
    #[serde(rename = "expirationTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

impl Into<NewSubscription> for EventsSubscriptionCreationRequestForCatalog {
    fn into(self) -> NewSubscription {
        NewSubscription {
            callback_address: self.callback_address,
            transfer_process: false,
            contract_negotiation_process: false,
            catalog: true,
            data_plane: false,
            active: true,
            expiration_time: self.expiration_time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventsSubscriptionCreationRequestForContractNegotiation {
    #[serde(rename = "callbackAddress")]
    pub callback_address: String,
    #[serde(rename = "expirationTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

impl Into<NewSubscription> for EventsSubscriptionCreationRequestForContractNegotiation {
    fn into(self) -> NewSubscription {
        NewSubscription {
            callback_address: self.callback_address,
            transfer_process: false,
            contract_negotiation_process: true,
            catalog: false,
            data_plane: false,
            active: true,
            expiration_time: self.expiration_time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventsSubscriptionCreationRequestForDataPlane {
    #[serde(rename = "callbackAddress")]
    pub callback_address: String,
    #[serde(rename = "expirationTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

impl Into<NewSubscription> for EventsSubscriptionCreationRequestForDataPlane {
    fn into(self) -> NewSubscription {
        NewSubscription {
            callback_address: self.callback_address,
            transfer_process: false,
            contract_negotiation_process: false,
            catalog: false,
            data_plane: true,
            active: true,
            expiration_time: self.expiration_time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum SubscriptionEntities {
    #[serde(rename = "TransferProcess")]
    TransferProcess,
    #[serde(rename = "Catalog")]
    Catalog,
    #[serde(rename = "ContractNegotiationProcess")]
    ContractNegotiationProcess,
    #[serde(rename = "DataPlaneProcess")]
    DataPlaneProcess,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventsSubscriptionCreationResponse {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: Urn,
    #[serde(rename = "callbackAddress")]
    pub callback_address: String,
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::NaiveDateTime,
    #[serde(rename = "expirationTime")]
    pub expiration_time: Option<chrono::NaiveDateTime>,
    #[serde(rename = "subscriptionEntity")]
    pub subscription_entity: SubscriptionEntities,
    #[serde(rename = "active")]
    pub active: bool,
}

impl TryFrom<subscription::Model> for EventsSubscriptionCreationResponse {
    type Error = Errors;

    fn try_from(value: subscription::Model) -> Outcome<Self> {
        let entity = match (
            value.transfer_process,
            value.contract_negotiation_process,
            value.data_plane,
            value.catalog,
        ) {
            (true, false, false, false) => SubscriptionEntities::TransferProcess,
            (false, true, false, false) => SubscriptionEntities::ContractNegotiationProcess,
            (false, false, true, false) => SubscriptionEntities::DataPlaneProcess,
            (false, false, false, true) => SubscriptionEntities::Catalog,
            _ => return Err(Errors::parse("Subscription Entity not valid", None)),
        };
        Ok(Self {
            subscription_id: get_urn_from_string(&value.id)?,
            callback_address: value.callback_address,
            timestamp: value.created_at,
            expiration_time: value.expiration_time,
            subscription_entity: entity,
            active: value.active,
        })
    }
}
