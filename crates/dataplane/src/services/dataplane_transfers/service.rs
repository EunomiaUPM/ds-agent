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

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use common::auth::access::{AccessScope, Rbac};
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{Page, Paginated, QueryFilter, Sort, MAX_BATCH_IDS};
use tracing::error;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::repo::dataplane_transfer::DataplaneTransfersRepo;
use crate::data::sea_orm::orm::dataplane_field::NewDataPlaneFieldModel;
use crate::data::sea_orm::orm::dataplane_transfer_logs::NewTransferLog;
use crate::data::sea_orm::orm::dataplane_transfers::{
    self as dataplane_transfers_model, EditDataplaneTransferModel, NewDataplaneTransfer,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, EditDataplaneTransferDto, NewDataplaneTransferDto,
};
use crate::entities::filters::DataplaneTransferFilter;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use common::cache::EntityCacheTrait;

pub struct DataplaneTransferService {
    data_plane_repo: Arc<dyn DataplaneRepoTrait>,
    cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
}

impl DataplaneTransferService {
    pub fn new(
        data_plane_repo: Arc<dyn DataplaneRepoTrait>,
        cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
    ) -> Self {
        Self {
            data_plane_repo,
            cache,
        }
    }

    fn repo(&self) -> Arc<dyn DataplaneTransfersRepo> {
        self.data_plane_repo.get_dataplane_transfers_repo()
    }

    async fn enrich_process(
        &self,
        process: dataplane_transfers_model::Model,
    ) -> Outcome<DataplaneTransferDto> {
        let process_urn = Urn::from_str(&process.id)?;

        let fields_models = self
            .data_plane_repo
            .get_dataplane_fields_repo()
            .get_all_dataplane_fields_by_process_id(&process.tenant_id, &process_urn)
            .await?;

        let fields = fields_models
            .into_iter()
            .map(|f| (f.key, f.value.unwrap_or_default()))
            .collect::<HashMap<String, String>>();

        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_dataplane_process_id(
                Some(process.tenant_id.clone()),
                &process_urn,
            )
            .await?;

        Ok(DataplaneTransferDto {
            inner: process,
            fields,
            logs,
        })
    }
}

#[async_trait::async_trait]
impl DataplaneTransferServiceTrait for DataplaneTransferService {
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &DataplaneTransferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DataplaneTransferDto>> {
        scope.require_read()?;
        filters.validate()?;

        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;

        let page = page.clamped();

        let repo = self.repo();
        let (transfers, total) = tokio::try_join!(
            repo.get_all_dataplane_transfers(&filters, &page, sort),
            repo.count_dataplane_transfers(&filters),
        )?;

        let mut items = Vec::with_capacity(transfers.len());
        for t in transfers {
            items.push(self.enrich_process(t).await?);
        }

        Ok(Paginated::from_page(items, &page, Some(total), |e| {
            Cursor::encode_sorted(
                &e.inner.created_at,
                &e.inner.updated_at.unwrap_or(e.inner.created_at),
                sort,
            )
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<DataplaneTransferDto> {
        scope.require_read()?;

        if let Some(cached) = self.cache.get_single(id).await? {
            if scope.permits(&cached.inner.tenant_id) {
                return Ok(cached);
            }
        }

        let process = self
            .repo()
            .get_dataplane_transfers_by_id(scope.tenant_filter().map(str::to_string), id)
            .await?
            .or_not_found(id, "dataplane transfer")?;

        let enriched = self.enrich_process(process).await?;
        let _ = self.cache.set_single(id, &enriched).await;
        Ok(enriched)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_by_process_id(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
    ) -> Outcome<DataplaneTransferDto> {
        scope.require_read()?;

        let process = self
            .repo()
            .get_by_transfer_process_id(scope.tenant_filter().map(str::to_string), process_id)
            .await?
            .or_not_found(process_id, "dataplane transfer")?;

        let enriched = self.enrich_process(process).await?;
        Ok(enriched)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn batch(
        &self,
        scope: &AccessScope,
        req: &BatchRequests,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        scope.require_read()?;

        if req.ids.len() > MAX_BATCH_IDS {
            return Err(Errors::format(
                BadFormat::Received,
                format!("batch request exceeds maximum of {MAX_BATCH_IDS} ids"),
                None,
            ));
        }

        if req.ids.is_empty() {
            return Ok(vec![]);
        }

        let processes = self
            .repo()
            .get_batch_dataplane_transfers(scope.tenant_filter().map(str::to_string), &req.ids)
            .await?;

        let mut dtos = Vec::with_capacity(processes.len());
        for p in processes {
            dtos.push(self.enrich_process(p).await?);
        }

        Ok(dtos)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto> {
        let mut cmd = cmd.clone();
        cmd.tenant_id = scope.resolve_create_tenant(Some(&cmd.tenant_id))?;

        let new_model: NewDataplaneTransfer = cmd.clone().into();
        let created_process = self.repo().create_dataplane_transfers(&new_model).await?;

        let log = NewTransferLog {
            tenant_id: created_process.tenant_id.clone(),
            dataplane_process_id: created_process.id.clone(),
            previous_state: None,
            new_state: cmd.state.clone(),
            trigger: "Creation".to_string(),
            reason: None,
        };
        if let Err(e) = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .create_log(log)
            .await
        {
            error!("Failed to create dataplane transfer log: {:?}", e);
        }

        let enriched = self.enrich_process(created_process).await?;

        if let Ok(urn) = Urn::from_str(&enriched.inner.id) {
            let _ = self.cache.set_single(&urn, &enriched).await;
        }

        Ok(enriched)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn edit(
        &self,
        scope: &AccessScope,
        id: &Urn,
        cmd: &EditDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto> {
        scope.require_write()?;

        // Authorize before writing anything; every row below belongs to the transfer's tenant.
        let tenant_id = self
            .repo()
            .get_dataplane_transfers_by_id(scope.tenant_filter().map(str::to_string), id)
            .await?
            .or_not_found(id, "dataplane transfer")?
            .tenant_id;

        if let Some(fields) = &cmd.fields {
            let fields_repo = self.data_plane_repo.get_dataplane_fields_repo();
            fields_repo
                .delete_all_dataplane_fields_by_process_id(&tenant_id, id)
                .await?;

            for (key, value) in fields {
                let new_field = NewDataPlaneFieldModel {
                    tenant_id: tenant_id.clone(),
                    key: key.clone(),
                    value: Some(value.clone()),
                };
                fields_repo
                    .create_dataplane_field(&tenant_id, id, &new_field)
                    .await?;
            }
        }

        let edit_model = EditDataplaneTransferModel {
            state: cmd.state.clone(),
            connector_instance_id: cmd.connector_instance_id.clone(),
            ingress_config: cmd.ingress_config.clone(),
            egress_config: cmd.egress_config.clone(),
            flow_control: cmd.flow_control.clone(),
        };

        let updated_process = self
            .repo()
            .put_dataplane_transfers(Some(tenant_id.clone()), id, &edit_model)
            .await?;

        if let Some(new_state) = &cmd.state {
            let log = NewTransferLog {
                tenant_id: updated_process.tenant_id.clone(),
                dataplane_process_id: updated_process.id.clone(),
                previous_state: None,
                new_state: new_state.clone(),
                trigger: "Update".to_string(),
                reason: None,
            };
            if let Err(e) = self
                .data_plane_repo
                .get_dataplane_transfer_logs_repo()
                .create_log(log)
                .await
            {
                error!("Failed to create dataplane transfer log: {:?}", e);
            }
        }

        let enriched = self.enrich_process(updated_process).await?;
        let _ = self.cache.set_single(id, &enriched).await;

        Ok(enriched)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        self.repo()
            .delete_dataplane_transfers(scope.tenant_filter().map(str::to_string), id)
            .await?;
        let _ = self.cache.delete_single(id).await;
        Ok(())
    }
}
