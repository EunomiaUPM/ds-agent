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

use std::sync::Arc;

use crate::data::entities::dataplane_transfers::{
    self, Column, EditDataplaneTransferModel, Entity as DataplaneTransferEntity,
    NewDataplaneTransferModel,
};
use crate::data::repo::dataplane_transfer::{DataplaneTransfersRepo, DataplaneTransfersRepoErrors};
use crate::entities::filters::DataplaneTransferFilter;
use common::paginated_spec::Cursor;
use common::query::{Page, Sort};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct DataplaneTransfersRepoForSql {
    db: Arc<DatabaseConnection>,
}

impl DataplaneTransfersRepoForSql {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn new_with_raw_db(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    fn fetch_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(Box::new(e)).into_errors()
    }

    fn decode_cursor(&self, cursor: &str) -> Outcome<chrono::DateTime<chrono::FixedOffset>> {
        Cursor::decode_timestamp(cursor)
            .map_err(|_| DataplaneTransfersRepoErrors::InvalidCursor.into_errors())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<DataplaneTransferEntity>,
        filters: &DataplaneTransferFilter,
    ) -> sea_orm::Select<DataplaneTransferEntity> {
        if let Some(tid) = &filters.tenant_id {
            q = q.filter(Column::TenantId.eq(tid.as_str()));
        }
        if let Some(pid) = &filters.transfer_process_id {
            q = q.filter(Column::TransferProcessId.eq(pid.as_str()));
        }
        if let Some(role) = &filters.role {
            q = q.filter(Column::Role.eq(role.clone()));
        }
        if let Some(mode) = &filters.interaction_mode {
            q = q.filter(Column::InteractionMode.eq(mode.clone()));
        }
        if let Some(state) = &filters.state {
            q = q.filter(Column::State.eq(state.clone()));
        }
        if let Some(after) = filters.created_after {
            q = q.filter(Column::CreatedAt.gt(after));
        }
        if let Some(before) = filters.created_before {
            q = q.filter(Column::CreatedAt.lt(before));
        }
        q
    }
}

#[async_trait::async_trait]
impl DataplaneTransfersRepo for DataplaneTransfersRepoForSql {
    async fn get_all_dataplane_transfers(
        &self,
        filters: &DataplaneTransferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<dataplane_transfers::Model>> {
        let mut q = Self::apply_base_filters(DataplaneTransferEntity::find(), filters);

        if let Some(cursor) = &page.cursor {
            let cursor_dt = self.decode_cursor(cursor)?;
            q = match sort {
                Sort::CreatedAtAsc => q.filter(Column::CreatedAt.gt(cursor_dt)),
                Sort::UpdatedAtAsc => q.filter(Column::UpdatedAt.gt(cursor_dt)),
                Sort::UpdatedAtDesc => q.filter(Column::UpdatedAt.lt(cursor_dt)),
                _ => q.filter(Column::CreatedAt.lt(cursor_dt)),
            };
        }

        q = match sort {
            Sort::CreatedAtAsc => q.order_by_asc(Column::CreatedAt).order_by_asc(Column::Id),
            Sort::UpdatedAtAsc => q.order_by_asc(Column::UpdatedAt).order_by_asc(Column::Id),
            Sort::UpdatedAtDesc => q.order_by_desc(Column::UpdatedAt).order_by_desc(Column::Id),
            _ => q.order_by_desc(Column::CreatedAt).order_by_desc(Column::Id),
        };

        let transfers = q
            .limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(transfers)
    }

    async fn count_dataplane_transfers(&self, filters: &DataplaneTransferFilter) -> Outcome<u64> {
        Self::apply_base_filters(DataplaneTransferEntity::find(), filters)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    async fn get_batch_dataplane_transfers(
        &self,
        tenant_id: &str,
        ids: &[Urn],
    ) -> Outcome<Vec<dataplane_transfers::Model>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let id_strings: Vec<String> = ids.iter().map(|urn| urn.to_string()).collect();
        let transfers = DataplaneTransferEntity::find()
            .filter(Column::Id.is_in(id_strings))
            .filter(Column::TenantId.eq(tenant_id))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(transfers)
    }

    async fn get_dataplane_transfers_by_id(
        &self,
        tenant_id: &str,
        process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>> {
        let transfer = DataplaneTransferEntity::find_by_id(process_id.to_string())
            .filter(Column::TenantId.eq(tenant_id))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(transfer)
    }

    async fn get_by_transfer_process_id(
        &self,
        tenant_id: &str,
        transfer_process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>> {
        let transfer = DataplaneTransferEntity::find()
            .filter(Column::TransferProcessId.eq(transfer_process_id.to_string()))
            .filter(Column::TenantId.eq(tenant_id))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(transfer)
    }

    async fn create_dataplane_transfers(
        &self,
        new_dataplane_transfer: &NewDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model> {
        let active_model: dataplane_transfers::ActiveModel = new_dataplane_transfer.clone().into();
        active_model.insert(self.db.as_ref()).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorCreatingDataplaneTransfer(Box::new(e)).into_errors()
        })
    }

    async fn put_dataplane_transfers(
        &self,
        tenant_id: &str,
        process_id: &Urn,
        new_dataplane_transfer: &EditDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model> {
        let existing = DataplaneTransferEntity::find_by_id(process_id.to_string())
            .filter(Column::TenantId.eq(tenant_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneTransfersRepoErrors::ErrorUpdatingDataplaneTransfer(Box::new(e))
                    .into_errors()
            })?
            .ok_or_else(|| DataplaneTransfersRepoErrors::DataplaneTransferNotFound.into_errors())?;

        let mut active_model: dataplane_transfers::ActiveModel = existing.into();
        active_model.updated_at = ActiveValue::Set(Some(chrono::Utc::now().into()));

        if let Some(state) = &new_dataplane_transfer.state {
            active_model.state = ActiveValue::Set(state.clone());
        }
        if let Some(cid) = &new_dataplane_transfer.connector_instance_id {
            active_model.connector_instance_id = ActiveValue::Set(Some(cid.to_string()));
        }
        if let Some(ingress) = &new_dataplane_transfer.ingress_config {
            active_model.ingress_config = ActiveValue::Set(ingress.clone());
        }
        if let Some(egress) = &new_dataplane_transfer.egress_config {
            active_model.egress_config = ActiveValue::Set(egress.clone());
        }
        if let Some(flow_control) = &new_dataplane_transfer.flow_control {
            active_model.flow_control = ActiveValue::Set(Some(flow_control.clone()));
        }

        active_model.update(self.db.as_ref()).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorUpdatingDataplaneTransfer(Box::new(e)).into_errors()
        })
    }

    async fn delete_dataplane_transfers(&self, tenant_id: &str, process_id: &Urn) -> Outcome<()> {
        let result = DataplaneTransferEntity::delete_many()
            .filter(Column::Id.eq(process_id.to_string()))
            .filter(Column::TenantId.eq(tenant_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneTransfersRepoErrors::ErrorDeletingDataplaneTransfer(Box::new(e))
                    .into_errors()
            })?;

        if result.rows_affected == 0 {
            return Err(DataplaneTransfersRepoErrors::DataplaneTransferNotFound.into_errors());
        }
        Ok(())
    }
}
