use super::{DataplaneTransferLogDto, DataplaneTransferLogsEntitiesTrait};
use crate::data::factory_trait::DataplaneRepoTrait;
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::Outcome;

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
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<DataplaneTransferLogDto>> {
        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_dataplane_process_id(&dataplane_process_id)
            .await?;

        Ok(logs.into_iter().map(|log| DataplaneTransferLogDto { inner: log }).collect())
    }
}
