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

pub(crate) mod persistence_protocol;
pub(crate) mod persistence_rpc;

use crate::entities::transfer_messages::{NewTransferMessageDto, TransferAgentMessagesTrait};
use crate::entities::transfer_process::{
    NewTransferProcessDto, TransferAgentProcessesTrait, TransferProcessDto,
};
use crate::protocols::dsp::protocol_types::{TransferProcessMessageTrait, TransferStateAttribute};
use crate::protocols::dsp::transfer_types::TransferState;
use std::collections::HashMap;
use std::format;
use std::sync::Arc;
use urn::Urn;

#[async_trait::async_trait]
#[allow(unused)]
pub trait TransferPersistenceTrait: Send + Sync {
    async fn get_transfer_process_service(
        &self,
    ) -> anyhow::Result<Arc<dyn TransferAgentProcessesTrait>>;
    async fn get_transfer_message_service(
        &self,
    ) -> anyhow::Result<Arc<dyn TransferAgentMessagesTrait>>;
    async fn fetch_process(&self, id: &str) -> anyhow::Result<TransferProcessDto>;
    async fn create_process(
        &self,
        protocol: &str,
        direction: &str,
        associated_agent_peer: &str,
        provider_pid: Option<Urn>,
        provider_address: Option<String>,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> anyhow::Result<TransferProcessDto>;
    async fn create_process_with_id(
        &self,
        id: Urn,
        protocol: &str,
        direction: &str,
        associated_agent_peer: &str,
        provider_pid: Option<Urn>,
        provider_address: Option<String>,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> anyhow::Result<TransferProcessDto>;
    async fn update_process(
        &self,
        id: &str,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> anyhow::Result<TransferProcessDto>;
}
