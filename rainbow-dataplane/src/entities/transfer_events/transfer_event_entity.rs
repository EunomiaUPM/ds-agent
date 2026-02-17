use crate::data::entities::transfer_event::NewTransferEvent;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::transfer_events::{
    NewTransferEventDto, TransferEventDto, TransferEventEntitiesTrait,
};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;

pub struct TransferEventEntityService {
    pub data_plane_repo: Arc<dyn DataplaneRepoTrait>,
}

impl TransferEventEntityService {
    pub fn new(data_plane_repo: &std::sync::Arc<dyn DataplaneRepoTrait>) -> Self {
        Self { data_plane_repo: data_plane_repo.clone() }
    }
}

#[async_trait::async_trait]
impl TransferEventEntitiesTrait for TransferEventEntityService {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<TransferEventDto>> {
        let events = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_all_transfer_events(limit, page)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(events.into_iter().map(|e| TransferEventDto { inner: e }).collect())
    }

    async fn get_batch_transfer_events(
        &self,
        ids: Vec<Urn>,
    ) -> anyhow::Result<Vec<TransferEventDto>> {
        let events = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_batch_transfer_events(&ids)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(events.into_iter().map(|e| TransferEventDto { inner: e }).collect())
    }

    async fn get_transfer_event_by_id(&self, id: &Urn) -> anyhow::Result<Option<TransferEventDto>> {
        let event = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_transfer_event_by_id(id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(event.map(|e| TransferEventDto { inner: e }))
    }

    async fn get_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<TransferEventDto>> {
        let events = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_all_transfer_events_by_process_id(process_id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(events.into_iter().map(|e| TransferEventDto { inner: e }).collect())
    }

    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEventDto,
    ) -> anyhow::Result<TransferEventDto> {
        let new_model: NewTransferEvent = new_transfer_event.clone().into();
        // Since transfer_id is in the DTO, we pass a dummy/default URN for the first arg of repo method if it still requires it,
        // OR better, we use the transfer_id from DTO to construct the URN expected by repo.
        // Wait, repo `create_transfer_event` takes `data_plane_process: &Urn`. Use a dummy URN as container for UUID.
        let urn_str = format!("urn:uuid:{}", new_transfer_event.transfer_id);
        let urn = urn::UrnBuilder::new("dummy", "id")
            .build()
            .unwrap_or_else(|_| "urn:dummy:id".parse::<urn::Urn>().unwrap());

        let created_event = self
            .data_plane_repo
            .get_transfer_events_repo()
            .create_transfer_event(&urn, &new_model)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(TransferEventDto { inner: created_event })
    }
}
