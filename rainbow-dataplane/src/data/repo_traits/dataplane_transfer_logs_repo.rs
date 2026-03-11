use crate::data::entities::dataplane_transfer_logs;
use anyhow::Error;
use async_trait::async_trait;
use thiserror::Error;
use urn::Urn;

#[async_trait]
pub trait DataplaneTransferLogsRepo: Send + Sync {
    async fn create_log(
        &self,
        new_log: dataplane_transfer_logs::NewTransferLog,
    ) -> Result<dataplane_transfer_logs::Model, DataplaneTransferLogsRepoErrors>;
    async fn get_all_transfer_logs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<Vec<dataplane_transfer_logs::Model>, DataplaneTransferLogsRepoErrors>;
    async fn get_transfer_log_by_id(
        &self,
        log_id: &Urn,
    ) -> Result<Option<dataplane_transfer_logs::Model>, DataplaneTransferLogsRepoErrors>;
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> Result<Vec<dataplane_transfer_logs::Model>, DataplaneTransferLogsRepoErrors>;
}

#[derive(Debug, Error)]
pub enum DataplaneTransferLogsRepoErrors {
    #[error("Dataplane transfer log not found")]
    DataplaneTransferLogNotFound,
    #[error("Error fetching dataplane transfer log. {0}")]
    ErrorFetchingDataplaneTransferLog(Error),
    #[error("Error creating dataplane transfer log. {0}")]
    ErrorCreatingDataplaneTransferLog(Error),
    #[error("Error deleting dataplane transfer log. {0}")]
    ErrorDeletingDataplaneTransferLog(Error),
    #[error("Error updating dataplane transfer log. {0}")]
    ErrorUpdatingDataplaneTransferLog(Error),
}
