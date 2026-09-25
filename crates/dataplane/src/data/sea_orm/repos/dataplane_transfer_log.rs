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

use sea_orm::QueryTrait;
use std::sync::Arc;

use crate::data::repo::dataplane_transfer_log::{
    DataplaneTransferLogsRepo, DataplaneTransferLogsRepoErrors,
};
use crate::data::sea_orm::orm::dataplane_transfer_logs::{
    self, Column, Entity as DataplaneTransferLogsEntity, NewTransferLog,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct DataplaneTransferLogsRepoForSql {
    db: Arc<DatabaseConnection>,
}

impl DataplaneTransferLogsRepoForSql {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn new_with_raw_db(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }
}

#[async_trait::async_trait]
impl DataplaneTransferLogsRepo for DataplaneTransferLogsRepoForSql {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_log(&self, new_log: NewTransferLog) -> Outcome<dataplane_transfer_logs::Model> {
        let active_model: dataplane_transfer_logs::ActiveModel = new_log.into();
        let result = active_model.insert(self.db.as_ref()).await;
        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(
                DataplaneTransferLogsRepoErrors::ErrorCreatingDataplaneTransferLog(Box::new(e))
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        tenant_id: Option<String>,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<dataplane_transfer_logs::Model>> {
        let logs = DataplaneTransferLogsEntity::find()
            .filter(Column::DataplaneProcessId.eq(dataplane_process_id.to_string()))
            .apply_if(tenant_id, |q, t| q.filter(Column::TenantId.eq(t)))
            .all(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneTransferLogsRepoErrors::ErrorFetchingDataplaneTransferLog(Box::new(e))
                    .into_errors()
            })?;

        Ok(logs)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_transfer_log_by_id(
        &self,
        tenant_id: &str,
        log_id: &Urn,
    ) -> Outcome<Option<dataplane_transfer_logs::Model>> {
        let log = DataplaneTransferLogsEntity::find_by_id(log_id.to_string())
            .filter(Column::TenantId.eq(tenant_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                DataplaneTransferLogsRepoErrors::ErrorFetchingDataplaneTransferLog(Box::new(e))
                    .into_errors()
            })?;

        Ok(log)
    }
}
