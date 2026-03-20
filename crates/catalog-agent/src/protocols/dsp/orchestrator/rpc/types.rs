/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::protocols::dsp::protocol_types::{
    CatalogErrorDto, CatalogMessageTrait, CatalogMessageType, CatalogMessageWrapper,
    CatalogRequestMessageDto, DatasetRequestMessage,
};
use crate::protocols::dsp::types::CatalogDspTraitDefinition;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::ContractRequestMessageOfferTypes;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;
use urn::Urn;

pub trait RpcCatalogMessageTrait: Debug + Sync + Send {
    fn get_associated_agent_peer(&self) -> Option<String>;
    fn get_filter_criterion(&self) -> Option<serde_json::Value>;
    fn get_dataset_id(&self) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcCatalogRequestMessageDto {
    associated_agent_peer: String,
    filter: serde_json::Value,
    #[serde(default)]
    pub no_cache: bool,
}

impl Into<CatalogMessageWrapper<CatalogRequestMessageDto>> for RpcCatalogRequestMessageDto {
    fn into(self) -> CatalogMessageWrapper<CatalogRequestMessageDto> {
        CatalogMessageWrapper {
            context: ContextField::default(),
            _type: CatalogMessageType::CatalogRequestMessage,
            dto: CatalogRequestMessageDto { filter: self.filter },
        }
    }
}

impl RpcCatalogMessageTrait for RpcCatalogRequestMessageDto {
    fn get_associated_agent_peer(&self) -> Option<String> {
        Some(self.associated_agent_peer.clone())
    }

    fn get_filter_criterion(&self) -> Option<serde_json::Value> {
        None
    }

    fn get_dataset_id(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcDatasetRequestMessageDto {
    associated_agent_peer: String,
    dataset: String,
    #[serde(default)]
    no_cache: Option<bool>,
}

impl Into<CatalogMessageWrapper<DatasetRequestMessage>> for RpcDatasetRequestMessageDto {
    fn into(self) -> CatalogMessageWrapper<DatasetRequestMessage> {
        let dataset_urn = Urn::from_str(&*self.dataset).unwrap();
        CatalogMessageWrapper {
            context: ContextField::default(),
            _type: CatalogMessageType::DatasetRequestMessage,
            dto: DatasetRequestMessage { dataset: dataset_urn },
        }
    }
}

impl RpcCatalogMessageTrait for RpcDatasetRequestMessageDto {
    fn get_associated_agent_peer(&self) -> Option<String> {
        Some(self.associated_agent_peer.clone())
    }

    fn get_filter_criterion(&self) -> Option<serde_json::Value> {
        None
    }

    fn get_dataset_id(&self) -> Option<String> {
        Some(self.dataset.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcCatalogResponseMessageDto<T, U>
where
    T: RpcCatalogMessageTrait,
    U: CatalogDspTraitDefinition,
{
    pub request: T,
    pub response: U,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RpcCatalogErrorDto<T>
where
    T: RpcCatalogMessageTrait,
{
    pub request: T,
    pub error: CatalogMessageWrapper<CatalogErrorDto>,
}
