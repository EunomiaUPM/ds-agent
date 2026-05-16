use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::services::transfer_process::views::TransferProcessView;
use common::batch_requests::BatchRequests;
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod service;
pub(crate) mod views;

#[async_trait::async_trait]
pub(crate) trait TransferProcessServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferProcessView>>;
    async fn get_one(&self, id: &Urn) -> Outcome<TransferProcessView>;
    async fn batch(&self, batch_request: &BatchRequests) -> Outcome<Vec<TransferProcessView>>;
    async fn create(
        &self,
        batch_request: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcessView>;
    async fn edit(
        &self,
        id: &Urn,
        cmd: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcessView>;
    async fn delete(&self, id: &Urn) -> Outcome<()>;
}
