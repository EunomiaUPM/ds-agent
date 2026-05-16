use crate::data::repo::transfer_message::TransferMessageRepoErrors;
use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Paginated, Sort, TransferMessageFilter};
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_message::views::TransferMessageView;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

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
    async fn get_all(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        let messages = self
            .message_repo
            .get_all_transfer_messages(filters, page, sort)
            .await?;

        let items = messages
            .into_iter()
            .map(TransferMessageView::assemble)
            .collect();
        Ok(Paginated {
            items,
            next_cursor: None,
            total: None,
        })
    }

    async fn get_all_by_process(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        let messages = self
            .message_repo
            .get_messages_by_process_id(process_id, filters, page, sort)
            .await?;

        let items = messages
            .into_iter()
            .map(TransferMessageView::assemble)
            .collect();
        Ok(Paginated {
            items,
            next_cursor: None,
            total: None,
        })
    }

    async fn get_one(&self, id: &Urn) -> Outcome<TransferMessageView> {
        let message = self
            .message_repo
            .get_transfer_message_by_id(id)
            .await?
            .ok_or_else(|| TransferMessageRepoErrors::TransferMessageNotFound.into_errors())?;

        Ok(TransferMessageView::assemble(message))
    }

    async fn create(&self, cmd: &NewTransferMessageCommand) -> Outcome<TransferMessageView> {
        let message = self.message_repo.create_transfer_message(cmd).await?;

        Ok(TransferMessageView::assemble(message))
    }

    async fn delete(&self, id: &Urn) -> Outcome<()> {
        self.message_repo.delete_transfer_message(id).await
    }
}
