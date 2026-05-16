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
use chrono::DateTime;

use crate::data::repo::transfer_process::TransferProcessRepoErrors;
use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::entities::transfer_process::TransferProcess;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::services::transfer_process::views::TransferProcessView;
use common::batch_requests::BatchRequests;
use std::collections::HashMap;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

fn encode_cursor(process: &TransferProcess, sort: &Sort) -> String {
    let dt: DateTime<chrono::Utc> = match sort {
        Sort::UpdatedAtDesc => process.updated_at(),
        _ => process.created_at(),
    };
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(dt.to_rfc3339())
}

pub(crate) struct TransferProcessService {
    process_repo: Arc<dyn TransferProcessRepoTrait>,
    identifiers_repo: Arc<dyn TransferIdentifierRepoTrait>,
}

impl TransferProcessService {
    pub fn new(
        process_repo: Arc<dyn TransferProcessRepoTrait>,
        identifiers_repo: Arc<dyn TransferIdentifierRepoTrait>,
    ) -> Self {
        Self {
            process_repo,
            identifiers_repo,
        }
    }
}

#[async_trait::async_trait]
impl TransferProcessServiceTrait for TransferProcessService {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_all(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferProcessView>> {
        let (processes, total) = tokio::try_join!(
            self.process_repo.get_all_transfer_processes(filters, page, sort),
            self.process_repo.count_transfer_processes(filters),
        )?;

        let urns: Vec<Urn> = processes.iter().map(|p| p.id().as_urn().clone()).collect();

        let raw_identifiers = self
            .identifiers_repo
            .get_identifiers_by_batch_process_id(&urns)
            .await?;

        let mut grouped: HashMap<Urn, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            grouped
                .entry(id.transfer_process_id)
                .or_default()
                .insert(id.key, id.value.unwrap_or_default());
        }
        let next_cursor = if processes.len() == page.limit as usize {
            processes.last().map(|p| encode_cursor(p, sort))
        } else {
            None
        };

        let items = processes
            .into_iter()
            .map(|p| {
                let extra = grouped.remove(p.id().as_urn()).unwrap_or_default();
                TransferProcessView::assemble(p, extra)
            })
            .collect();

        Ok(Paginated {
            items,
            next_cursor,
            total: Some(total),
        })
    }

    #[tracing::instrument(level = "info", skip(self), fields(id = %id), err)]
    async fn get_one(&self, id: &Urn) -> Outcome<TransferProcessView> {
        let process = self
            .process_repo
            .get_transfer_process_by_id(id)
            .await?
            .ok_or_else(|| TransferProcessRepoErrors::TransferProcessNotFound.into_errors())?;

        let raw_identifiers = self
            .identifiers_repo
            .get_identifiers_by_process_id(id)
            .await?;

        let extra: HashMap<String, String> = raw_identifiers
            .into_iter()
            .filter_map(|i| i.value.map(|v| (i.key, v)))
            .collect();

        Ok(TransferProcessView::assemble(process, extra))
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn batch(&self, batch_request: &BatchRequests) -> Outcome<Vec<TransferProcessView>> {
        let processes = self
            .process_repo
            .get_batch_transfer_processes(&batch_request.ids)
            .await?;

        let urns: Vec<Urn> = processes.iter().map(|p| p.id().as_urn().clone()).collect();

        let raw_identifiers = self
            .identifiers_repo
            .get_identifiers_by_batch_process_id(&urns)
            .await?;

        let mut grouped: HashMap<Urn, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            grouped
                .entry(id.transfer_process_id)
                .or_default()
                .insert(id.key, id.value.unwrap_or_default());
        }

        let views = processes
            .into_iter()
            .map(|p| {
                let extra = grouped.remove(p.id().as_urn()).unwrap_or_default();
                TransferProcessView::assemble(p, extra)
            })
            .collect();

        Ok(views)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(&self, cmd: &NewTransferProcessCommand) -> Outcome<TransferProcessView> {
        let process = self.process_repo.create_transfer_process(cmd).await?;

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

        let extra: HashMap<String, String> = cmd.identifiers.clone().unwrap_or_default();

        Ok(TransferProcessView::assemble(process, extra))
    }

    #[tracing::instrument(level = "info", skip(self, cmd), fields(id = %id), err)]
    async fn edit(
        &self,
        id: &Urn,
        cmd: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcessView> {
        let process = self.process_repo.put_transfer_process(id, cmd).await?;

        if let Some(identifiers) = &cmd.identifiers {
            for (key, value) in identifiers {
                let identifier =
                    TransferProcessIdentifier::new(id.clone(), key.clone(), Some(value.clone()));
                self.identifiers_repo
                    .upsert_identifier(id, &identifier)
                    .await?;
            }
        }

        let raw_identifiers = self
            .identifiers_repo
            .get_identifiers_by_process_id(id)
            .await?;

        let extra: HashMap<String, String> = raw_identifiers
            .into_iter()
            .filter_map(|i| i.value.map(|v| (i.key, v)))
            .collect();

        Ok(TransferProcessView::assemble(process, extra))
    }

    #[tracing::instrument(level = "info", skip(self), fields(id = %id), err)]
    async fn delete(&self, id: &Urn) -> Outcome<()> {
        self.process_repo.delete_transfer_process(id).await
    }
}
