use crate::data::repo_traits::dataplane_transfer_logs_repo::DataplaneTransferLogsRepo;
use crate::data::{
    entities::dataplane_transfer_logs,
    repo_traits::dataplane_transfer_logs_repo::DataplaneTransferLogsRepoErrors,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct DataplaneTransferLogsRepoForSql {
    db: DatabaseConnection,
}

impl DataplaneTransferLogsRepoForSql {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl DataplaneTransferLogsRepo for DataplaneTransferLogsRepoForSql {
    async fn create_log(
        &self,
        new_log: dataplane_transfer_logs::NewTransferLog,
    ) -> Outcome<dataplane_transfer_logs::Model> {
        let active_model: dataplane_transfer_logs::ActiveModel = new_log.into();
        let result = active_model.insert(&self.db).await;
        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(DataplaneTransferLogsRepoErrors::ErrorCreatingDataplaneTransferLog(
                e.into(),
            )
            .into_errors()),
        }
    }

    async fn get_all_transfer_logs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataplane_transfer_logs::Model>> {
        let logs = dataplane_transfer_logs::Entity::find()
            .limit(limit.unwrap_or(100))
            .offset(page.map(|p| p * limit.unwrap_or(100)).unwrap_or(0))
            .all(&self.db)
            .await;
        match logs {
            Ok(logs) => Ok(logs),
            Err(e) => Err(DataplaneTransferLogsRepoErrors::ErrorFetchingDataplaneTransferLog(
                e.into(),
            )
            .into_errors()),
        }
    }

    async fn get_transfer_log_by_id(
        &self,
        log_id: &Urn,
    ) -> Outcome<Option<dataplane_transfer_logs::Model>> {
        let log_id = log_id.to_string();
        let log = dataplane_transfer_logs::Entity::find_by_id(log_id).one(&self.db).await;
        match log {
            Ok(log) => Ok(log),
            Err(e) => Err(DataplaneTransferLogsRepoErrors::ErrorFetchingDataplaneTransferLog(
                e.into(),
            )
            .into_errors()),
        }
    }

    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<dataplane_transfer_logs::Model>> {
        let dataplane_process_id = dataplane_process_id.to_string();
        let logs = dataplane_transfer_logs::Entity::find()
            .filter(dataplane_transfer_logs::Column::DataplaneProcessId.eq(dataplane_process_id))
            .all(&self.db)
            .await;
        match logs {
            Ok(logs) => Ok(logs),
            Err(e) => Err(DataplaneTransferLogsRepoErrors::ErrorFetchingDataplaneTransferLog(
                e.into(),
            )
            .into_errors()),
        }
    }
}
