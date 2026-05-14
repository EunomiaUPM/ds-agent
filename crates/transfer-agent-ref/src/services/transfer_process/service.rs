use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
use common::batch_requests::BatchRequests;
use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::services::transfer_process::views::TransferProcessView;

pub(crate) struct TransferProcessService {
    repo: Arc<dyn TransferProcessRepoTrait>,
}

#[async_trait::async_trait]
impl TransferProcessServiceTrait for TransferProcessService {
    async fn get_all(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferProcessView>> {
        todo!()
    }

    async fn get_one(&self, id: &Urn) -> Outcome<TransferProcessView> {
        todo!()
    }

    async fn batch(&self, batch_request: &BatchRequests) -> Outcome<Vec<TransferProcessView>> {
        todo!()
    }

    async fn create(&self, cmd: &NewTransferProcessCommand) -> Outcome<TransferProcessView> {
        todo!()
    }

    async fn edit(&self, id: &Urn, cmd: &EditTransferProcessCommand) -> Outcome<TransferProcessView> {
        todo!()
    }

    async fn delete(&self, id: &Urn) -> Outcome<()> {
        todo!()
    }
}