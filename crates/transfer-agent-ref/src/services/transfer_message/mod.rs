pub(crate) mod views;
pub(crate) mod service;

use urn::Urn;
use ymir::errors::Outcome;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Paginated, Sort, TransferMessageFilter};
use crate::services::transfer_message::views::TransferMessageView;

#[async_trait::async_trait]
pub(crate) trait TransferMessageServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>>;

    async fn get_all_by_process(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>>;

    async fn get_one(&self, id: &Urn) -> Outcome<TransferMessageView>;

    async fn create(&self, cmd: &NewTransferMessageCommand) -> Outcome<TransferMessageView>;

    async fn delete(&self, id: &Urn) -> Outcome<()>;
}