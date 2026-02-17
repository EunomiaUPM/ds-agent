use super::{DataplaneTransferLogDto, DataplaneTransferLogsEntitiesTrait};
use crate::data::factory_trait::DataplaneRepoTrait;
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub struct DataplaneTransferLogsEntityService {
    pub data_plane_repo: Arc<dyn DataplaneRepoTrait>,
}

impl DataplaneTransferLogsEntityService {
    pub fn new(data_plane_repo: Arc<dyn DataplaneRepoTrait>) -> Self {
        Self { data_plane_repo }
    }
}

#[async_trait::async_trait]
impl DataplaneTransferLogsEntitiesTrait for DataplaneTransferLogsEntityService {
    async fn get_transfer_logs_by_transfer_id(
        &self,
        transfer_id: Uuid,
    ) -> anyhow::Result<Vec<DataplaneTransferLogDto>> {
        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_transfer_id(&transfer_id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(logs.into_iter().map(|log| DataplaneTransferLogDto { inner: log }).collect())
    }
}
