use crate::data::entities::dataplane_transfer_logs;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
#[async_trait]
pub trait DataplaneTransferLogsRepo: Send + Sync {
    async fn create_log(
        &self,
        new_log: dataplane_transfer_logs::NewTransferLog,
    ) -> Result<dataplane_transfer_logs::Model>;
    async fn get_all_transfer_logs(&self) -> Result<Vec<dataplane_transfer_logs::Model>>;
    async fn get_transfer_log_by_id(
        &self,
        log_id: &uuid::Uuid,
    ) -> Result<Option<dataplane_transfer_logs::Model>>;
    async fn get_transfer_logs_by_transfer_id(
        &self,
        transfer_id: &uuid::Uuid,
    ) -> Result<Vec<dataplane_transfer_logs::Model>>;
}
