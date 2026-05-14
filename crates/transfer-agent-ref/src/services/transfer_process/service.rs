use std::collections::HashMap;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
use common::batch_requests::BatchRequests;
use crate::data::repo::transfer_process::TransferProcessRepoErrors;
use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::services::transfer_process::views::TransferProcessView;
use ymir::errors::RepoIntoErrors;

pub(crate) struct TransferProcessService {
    process_repo: Arc<dyn TransferProcessRepoTrait>,
    identifiers_repo: Arc<dyn TransferIdentifierRepoTrait>,
}

impl TransferProcessService {
    pub fn new(
        process_repo: Arc<dyn TransferProcessRepoTrait>,
        identifiers_repo: Arc<dyn TransferIdentifierRepoTrait>,
    ) -> Self {
        Self { process_repo, identifiers_repo }
    }
}

#[async_trait::async_trait]
impl TransferProcessServiceTrait for TransferProcessService {
    async fn get_all(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferProcessView>> {
        let processes = self.process_repo
            .get_all_transfer_processes(filters, page, sort)
            .await?;

        let urns: Vec<Urn> = processes.iter()
            .map(|p| p.id().as_urn().clone())
            .collect();

        let raw_identifiers = self.identifiers_repo
            .get_identifiers_by_batch_process_id(&urns)
            .await?;

        let mut grouped: HashMap<String, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            grouped
                .entry(id.transfer_process_id.to_string())
                .or_default()
                .insert(id.key, id.value.unwrap_or_default());
        }

        let items = processes
            .into_iter()
            .map(|p| {
                let extra = grouped.remove(&p.id().to_string()).unwrap_or_default();
                TransferProcessView::assemble(p, extra)
            })
            .collect();

        Ok(Paginated { items, next_cursor: None, total: None })
    }

    async fn get_one(&self, id: &Urn) -> Outcome<TransferProcessView> {
        let process = self.process_repo
            .get_transfer_process_by_id(id)
            .await?
            .ok_or_else(|| TransferProcessRepoErrors::TransferProcessNotFound.into_errors())?;

        let raw_identifiers = self.identifiers_repo
            .get_identifiers_by_process_id(id)
            .await?;

        let extra: HashMap<String, String> = raw_identifiers
            .into_iter()
            .filter_map(|i| i.value.map(|v| (i.key, v)))
            .collect();

        Ok(TransferProcessView::assemble(process, extra))
    }

    async fn batch(&self, batch_request: &BatchRequests) -> Outcome<Vec<TransferProcessView>> {
        let processes = self.process_repo
            .get_batch_transfer_processes(&batch_request.ids)
            .await?;

        let urns: Vec<Urn> = processes.iter()
            .map(|p| p.id().as_urn().clone())
            .collect();

        let raw_identifiers = self.identifiers_repo
            .get_identifiers_by_batch_process_id(&urns)
            .await?;

        let mut grouped: HashMap<String, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            grouped
                .entry(id.transfer_process_id.to_string())
                .or_default()
                .insert(id.key, id.value.unwrap_or_default());
        }

        let views = processes
            .into_iter()
            .map(|p| {
                let extra = grouped.remove(&p.id().to_string()).unwrap_or_default();
                TransferProcessView::assemble(p, extra)
            })
            .collect();

        Ok(views)
    }

    async fn create(&self, cmd: &NewTransferProcessCommand) -> Outcome<TransferProcessView> {
        let process = self.process_repo
            .create_transfer_process(cmd)
            .await?;

        if let Some(identifiers) = &cmd.identifiers {
            for (key, value) in identifiers {
                let identifier = TransferProcessIdentifier::new(
                    process.id().as_urn().clone(),
                    key.clone(),
                    Some(value.clone()),
                );
                self.identifiers_repo
                    .upsert_identifier(process.id().as_urn(), &identifier)
                    .await?;
            }
        }

        let extra: HashMap<String, String> = cmd.identifiers
            .as_ref()
            .map(|ids| ids.clone())
            .unwrap_or_default();

        Ok(TransferProcessView::assemble(process, extra))
    }

    async fn edit(&self, id: &Urn, cmd: &EditTransferProcessCommand) -> Outcome<TransferProcessView> {
        let process = self.process_repo
            .put_transfer_process(id, cmd)
            .await?;

        if let Some(identifiers) = &cmd.identifiers {
            for (key, value) in identifiers {
                let identifier = TransferProcessIdentifier::new(
                    id.clone(),
                    key.clone(),
                    Some(value.clone()),
                );
                self.identifiers_repo
                    .upsert_identifier(id, &identifier)
                    .await?;
            }
        }

        let raw_identifiers = self.identifiers_repo
            .get_identifiers_by_process_id(id)
            .await?;

        let extra: HashMap<String, String> = raw_identifiers
            .into_iter()
            .filter_map(|i| i.value.map(|v| (i.key, v)))
            .collect();

        Ok(TransferProcessView::assemble(process, extra))
    }

    async fn delete(&self, id: &Urn) -> Outcome<()> {
        self.process_repo.delete_transfer_process(id).await
    }
}