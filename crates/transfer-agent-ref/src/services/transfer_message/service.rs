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

use base64::Engine;

use crate::data::repo::transfer_message::TransferMessageRepoErrors;
use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Paginated, Sort, TransferMessageFilter};
use crate::entities::transfer_message::TransferMessage;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_message::views::TransferMessageView;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

fn encode_cursor(msg: &TransferMessage) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(msg.occurred_at().to_rfc3339())
}

pub(crate) struct TransferMessageService {
    message_repo: Arc<dyn TransferMessageRepoTrait>,
}

impl TransferMessageService {
    pub fn new(message_repo: Arc<dyn TransferMessageRepoTrait>) -> Self {
        Self { message_repo }
    }
}

#[async_trait::async_trait]
impl TransferMessageServiceTrait for TransferMessageService {
    #[tracing::instrument(skip_all, err)]
    async fn get_all(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        let (messages, total) = tokio::try_join!(
            self.message_repo.get_all_transfer_messages(filters, page, sort),
            self.message_repo.count_transfer_messages(filters),
        )?;
        let next_cursor = if messages.len() == page.limit as usize {
            messages.last().map(encode_cursor)
        } else {
            None
        };
        let items = messages.into_iter().map(TransferMessageView::assemble).collect();
        Ok(Paginated { items, next_cursor, total: Some(total) })
    }

    #[tracing::instrument(skip(self, filters, page, sort), fields(process_id = %process_id), err)]
    async fn get_all_by_process(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        let (messages, total) = tokio::try_join!(
            self.message_repo.get_messages_by_process_id(process_id, filters, page, sort),
            self.message_repo.count_transfer_messages(filters),
        )?;
        let next_cursor = if messages.len() == page.limit as usize {
            messages.last().map(encode_cursor)
        } else {
            None
        };
        let items = messages.into_iter().map(TransferMessageView::assemble).collect();
        Ok(Paginated { items, next_cursor, total: Some(total) })
    }

    #[tracing::instrument(skip(self), fields(id = %id), err)]
    async fn get_one(&self, id: &Urn) -> Outcome<TransferMessageView> {
        let message = self
            .message_repo
            .get_transfer_message_by_id(id)
            .await?
            .ok_or_else(|| TransferMessageRepoErrors::TransferMessageNotFound.into_errors())?;

        Ok(TransferMessageView::assemble(message))
    }

    #[tracing::instrument(skip_all, err)]
    async fn create(&self, cmd: &NewTransferMessageCommand) -> Outcome<TransferMessageView> {
        let message = self.message_repo.create_transfer_message(cmd).await?;

        Ok(TransferMessageView::assemble(message))
    }

    #[tracing::instrument(skip(self), fields(id = %id), err)]
    async fn delete(&self, id: &Urn) -> Outcome<()> {
        self.message_repo.delete_transfer_message(id).await
    }
}
