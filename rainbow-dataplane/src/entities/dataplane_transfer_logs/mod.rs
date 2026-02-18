use crate::data::entities::dataplane_transfer_logs;
use serde::{Deserialize, Serialize};
use urn::Urn;
use uuid::Uuid;

pub mod dataplane_transfer_logs_entity;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataplaneTransferLogDto {
    #[serde(flatten)]
    pub inner: dataplane_transfer_logs::Model,
}

#[async_trait::async_trait]
pub trait DataplaneTransferLogsEntitiesTrait: Send + Sync + 'static {
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> anyhow::Result<Vec<DataplaneTransferLogDto>>;
}
