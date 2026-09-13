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

use crate::data::entities::transfer_message::NewTransferMessageModel;
use crate::data::factory_trait::TransferAgentRepoTrait;
use crate::entities::transfer_messages::{
    NewTransferMessageDto, TransferAgentMessagesTrait, TransferMessageDto,
};
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct TransferAgentMessagesService {
    pub transfer_repo: Arc<dyn TransferAgentRepoTrait>,
}

impl TransferAgentMessagesService {
    pub fn new(transfer_repo: Arc<dyn TransferAgentRepoTrait>) -> Self {
        Self { transfer_repo }
    }
}

use crate::entities::filters::TransferMessageFilter;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};

#[async_trait::async_trait]
impl TransferAgentMessagesTrait for TransferAgentMessagesService {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<Paginated<TransferMessageDto>> {
        let (messages, total) = self
            .transfer_repo
            .get_transfer_message_repo()
            .get_all_transfer_messages(filters, page, sort)
            .await?;

        let items: Vec<_> = messages
            .into_iter()
            .map(|m| TransferMessageDto { inner: m })
            .collect();

        Ok(Paginated::from_page(items, page, total, |last| {
            Cursor::encode_composite(&last.inner.created_at, &last.inner.id)
        }))
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferMessageDto>> {
        let messages = self
            .transfer_repo
            .get_transfer_message_repo()
            .get_messages_by_process_id(process_id)
            .await?;

        Ok(messages
            .into_iter()
            .map(|m| TransferMessageDto { inner: m })
            .collect())
    }

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<TransferMessageDto> {
        let message = self
            .transfer_repo
            .get_transfer_message_repo()
            .get_transfer_message_by_id(id)
            .await?
            .ok_or_else(|| Errors::crazy("Transfer Message not found", None))?;

        Ok(TransferMessageDto { inner: message })
    }

    async fn create_transfer_message(
        &self,
        new_model_dto: &NewTransferMessageDto,
    ) -> Outcome<TransferMessageDto> {
        let new_model: NewTransferMessageModel = new_model_dto.clone().into();

        let created = self
            .transfer_repo
            .get_transfer_message_repo()
            .create_transfer_message(&new_model)
            .await?;

        Ok(TransferMessageDto { inner: created })
    }

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()> {
        self.transfer_repo
            .get_transfer_message_repo()
            .delete_transfer_message(id)
            .await?;
        Ok(())
    }
}
