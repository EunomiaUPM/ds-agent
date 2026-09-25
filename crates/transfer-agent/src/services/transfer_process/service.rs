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

use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{MAX_BATCH_IDS, Page, Paginated, QueryFilter, Sort};
use std::collections::HashMap;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::filters::TransferProcessFilter;
use crate::entities::transfer_process::TransferProcess;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::services::transfer_process::views::TransferProcessView;

pub(crate) struct TransferProcessService {
    process_repo: Arc<dyn TransferProcessRepoTrait>,
    identifiers_repo: Arc<dyn TransferIdentifierRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl TransferProcessService {
    pub fn new(
        process_repo: Arc<dyn TransferProcessRepoTrait>,
        identifiers_repo: Arc<dyn TransferIdentifierRepoTrait>,
    ) -> Self {
        Self {
            process_repo,
            identifiers_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl TransferProcessServiceTrait for TransferProcessService {
    /// Get all TransferProcess services
    /// Validate filtering based in tenant-id
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferProcessView>> {
        scope.require_read()?;
        filters.validate()?;
        // tenant filters
        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;
        // pagination
        let page = page.clamped();
        // get entities
        let (processes, total) = tokio::try_join!(
            self.process_repo
                .get_all_transfer_processes(&filters, &page, sort),
            self.process_repo.count_transfer_processes(&filters),
        )?;
        // get urns
        let urns: Vec<Urn> = processes.iter().map(|p| p.id().as_urn().clone()).collect();
        // get extra identifiers
        let raw_identifiers = if urns.is_empty() {
            vec![]
        } else {
            self.identifiers_repo
                .get_identifiers_by_batch_process_id(&urns)
                .await?
        };
        // zip extra identifiers into entities
        let mut grouped: HashMap<Urn, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            grouped
                .entry(id.transfer_process_id)
                .or_default()
                .insert(id.key, id.value.unwrap_or_default());
        }
        // assemble transfer process view
        let items = processes
            .into_iter()
            .map(|p| {
                let extra = grouped.remove(p.id().as_urn()).unwrap_or_default();
                TransferProcessView::assemble(p, extra)
            })
            .collect();
        Ok(Paginated::from_page(items, &page, Some(total), |p| {
            Cursor::encode_sorted(&p.created_at, &p.updated_at, sort)
        }))
    }

    /// Get single transfer process entity
    /// If tenant-id is coincident ok, otherwise not_found
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(tenant = %scope.acting_tenant(), id = %id)
    )]
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<TransferProcessView> {
        scope.require_read()?;
        let process = self
            .process_repo
            .get_transfer_process_by_id(scope.tenant_filter().map(str::to_string), id)
            .await?
            .or_not_found(id, "transfer process")?;

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

    /// Batch transfer processes
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn batch(
        &self,
        scope: &AccessScope,
        batch_request: &BatchRequests,
    ) -> Outcome<Vec<TransferProcessView>> {
        scope.require_read()?;
        // Validate max batch
        if batch_request.ids.len() > MAX_BATCH_IDS {
            return Err(Errors::format(
                BadFormat::Received,
                format!("batch request exceeds the maximum of {MAX_BATCH_IDS} ids"),
                None,
            ));
        }
        if batch_request.ids.is_empty() {
            return Ok(vec![]);
        }
        // Get data from db filtered by tenant
        let processes = self
            .process_repo
            .get_batch_transfer_processes(
                scope.tenant_filter().map(str::to_string),
                &batch_request.ids,
            )
            .await?;
        // Fetch extra identifiers
        let urns: Vec<Urn> = processes.iter().map(|p| p.id().as_urn().clone()).collect();
        let raw_identifiers = if urns.is_empty() {
            vec![]
        } else {
            self.identifiers_repo
                .get_identifiers_by_batch_process_id(&urns)
                .await?
        };
        // Zip extra identifiers
        let mut grouped: HashMap<Urn, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            grouped
                .entry(id.transfer_process_id)
                .or_default()
                .insert(id.key, id.value.unwrap_or_default());
        }
        // Assemble TransferProcessView
        let views = processes
            .into_iter()
            .map(|p| {
                let extra = grouped.remove(p.id().as_urn()).unwrap_or_default();
                TransferProcessView::assemble(p, extra)
            })
            .collect();
        Ok(views)
    }

    /// Create a new transfer process entity
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcessView> {
        let mut cmd = cmd.clone();
        cmd.tenant_id = Some(scope.resolve_create_tenant(cmd.tenant_id.as_deref())?);
        // Create in db
        let process = self.process_repo.create_transfer_process(&cmd).await?;
        // if extra identifiers in cmd upsert identifiers
        if let Some(identifiers) = &cmd.identifiers {
            for (key, value) in identifiers {
                let identifier = TransferProcessIdentifier::with_tenant(
                    process.tenant_id(),
                    process.id().as_urn().clone(),
                    key.clone(),
                    Some(value.clone()),
                );
                self.identifiers_repo
                    .upsert_identifier(process.id().as_urn(), &identifier)
                    .await?;
            }
        }
        // Zip identifiers
        let extra: HashMap<String, String> = cmd.identifiers.clone().unwrap_or_default();
        // Assemble and serve view
        let view = TransferProcessView::assemble(process, extra);
        events::emit_action!(
            self.event_bus,
            &view.tenant_id,
            crate::EVENT_PREFIX,
            "process",
            "create",
            &view
        );
        Ok(view)
    }

    /// Edit a transfer process
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn edit(
        &self,
        scope: &AccessScope,
        id: &Urn,
        cmd: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcessView> {
        scope.require_write()?;
        // Hit db
        let process = self
            .process_repo
            .put_transfer_process(scope.tenant_filter().map(str::to_string), id, cmd)
            .await?;
        // if extra identifiers in cmd upsert identifiers
        if let Some(identifiers) = &cmd.identifiers {
            for (key, value) in identifiers {
                let identifier = TransferProcessIdentifier::with_tenant(
                    process.tenant_id(),
                    id.clone(),
                    key.clone(),
                    Some(value.clone()),
                );
                self.identifiers_repo
                    .upsert_identifier(id, &identifier)
                    .await?;
            }
        }
        // if extra identifiers in cmd upsert identifiers
        let raw_identifiers = self
            .identifiers_repo
            .get_identifiers_by_process_id(id)
            .await?;
        // Zip identifiers
        let extra: HashMap<String, String> = raw_identifiers
            .into_iter()
            .filter_map(|i| i.value.map(|v| (i.key, v)))
            .collect();
        // Assemble and serve view
        let view = TransferProcessView::assemble(process, extra);
        events::emit_action!(
            self.event_bus,
            &view.tenant_id,
            crate::EVENT_PREFIX,
            "process",
            "edit",
            &view
        );
        Ok(view)
    }

    /// Delete a transfer process
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(tenant = %scope.acting_tenant(), id = %id)
    )]
    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        // Hit db
        let owner = self
            .process_repo
            .delete_transfer_process(scope.tenant_filter().map(str::to_string), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            &owner,
            crate::EVENT_PREFIX,
            "process",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
