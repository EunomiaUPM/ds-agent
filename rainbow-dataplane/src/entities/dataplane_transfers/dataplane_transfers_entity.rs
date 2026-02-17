use crate::data::entities::dataplane_transfers::{
    self as dataplane_transfers_model, EditDataplaneTransferModel, NewDataplaneTransfer,
    TransferState,
};
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::repo_traits::dataplane_transfers_repo::DataplaneTransfersRepo;
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    NewDataplaneTransferDto,
};
use rainbow_common::errors::{helpers::BadFormat, CommonErrors, ErrorLog};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use uuid::Uuid;

use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;

pub struct DataplaneTransfersEntityService {
    pub data_plane_repo: Arc<dyn DataplaneRepoTrait>,
    pub cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
}

impl DataplaneTransfersEntityService {
    pub fn new(
        data_plane_repo: Arc<dyn DataplaneRepoTrait>,
        cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
    ) -> Self {
        Self { data_plane_repo, cache }
    }

    async fn enrich_process(
        &self,
        process: dataplane_transfers_model::Model,
    ) -> anyhow::Result<DataplaneTransferDto> {
        let process_urn = Urn::from_str(&format!("urn:uuid:{}", process.id)).map_err(|e| {
            let err = CommonErrors::format_new(BadFormat::Unknown, &format!("Invalid URN: {}", e));
            error!("{}", err.log());
            err
        })?;

        let fields = self
            .data_plane_repo
            .get_dataplane_fields_repo()
            .get_all_dataplane_fields_by_process_id(&process_urn)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_transfer_id(&process.id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(DataplaneTransferDto { inner: process, fields, logs })
    }
}

#[async_trait::async_trait]
impl DataplaneTransfersEntitiesTrait for DataplaneTransfersEntityService {
    async fn get_all_dataplane_transfers(&self) -> anyhow::Result<Vec<DataplaneTransferDto>> {
        let transfers = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_all_dataplane_transfers(None, None)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        let mut dtos = Vec::with_capacity(transfers.len());
        for t in transfers {
            let dto = self.enrich_process(t).await?;
            dtos.push(dto);
        }

        Ok(dtos)
    }

    async fn get_dataplane_transfer_by_id(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Option<DataplaneTransferDto>> {
        let urn_str = format!("urn:uuid:{}", id);
        let urn = Urn::from_str(&urn_str)
            .map_err(|e| CommonErrors::format_new(BadFormat::Unknown, &e.to_string()))?;

        // 1. Try cache
        if let Some(cached) = self.cache.get_single(&urn).await? {
            return Ok(Some(cached));
        }

        // 2. Try DB
        let process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_dataplane_transfers_by_id(&urn)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        if let Some(p) = process {
            let enriched = self.enrich_process(p).await?;
            // 3. Update cache
            let _ = self.cache.set_single(&urn, &enriched).await;
            Ok(Some(enriched))
        } else {
            Ok(None)
        }
    }

    async fn get_dataplane_transfer_by_process_id(
        &self,
        process_id: &str,
    ) -> anyhow::Result<Option<DataplaneTransferDto>> {
        let process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_by_transfer_process_id(process_id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        if let Some(p) = process {
            Ok(Some(self.enrich_process(p).await?))
        } else {
            Ok(None)
        }
    }

    async fn create_dataplane_transfer(
        &self,
        new_data_plane_process: &NewDataplaneTransferDto,
    ) -> anyhow::Result<DataplaneTransferDto> {
        let new_model: NewDataplaneTransfer = new_data_plane_process.clone().into();
        let created_process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .create_dataplane_transfers(&new_model)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        let enriched = self.enrich_process(created_process).await?;

        // Cache update
        let urn_str = format!("urn:uuid:{}", enriched.inner.id);
        if let Ok(urn) = Urn::from_str(&urn_str) {
            let _ = self.cache.set_single(&urn, &enriched).await;
        }

        Ok(enriched)
    }

    async fn update_state(
        &self,
        id: Uuid,
        state: TransferState,
    ) -> anyhow::Result<DataplaneTransferDto> {
        let urn_str = format!("urn:uuid:{}", id);
        let urn = Urn::from_str(&urn_str)
            .map_err(|e| CommonErrors::format_new(BadFormat::Unknown, &e.to_string()))?;

        let edit_model = EditDataplaneTransferModel { state: Some(state), flow_control: None };

        let updated_process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .put_dataplane_transfers(&urn, &edit_model)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;
        let enriched = self.enrich_process(updated_process).await?;

        // Update cache
        let _ = self.cache.set_single(&urn, &enriched).await;

        Ok(enriched)
    }

    async fn update_flow_control(
        &self,
        id: Uuid,
        flow_control: Value,
    ) -> anyhow::Result<DataplaneTransferDto> {
        let urn_str = format!("urn:uuid:{}", id);
        let urn = Urn::from_str(&urn_str)
            .map_err(|e| CommonErrors::format_new(BadFormat::Unknown, &e.to_string()))?;

        let edit_model =
            EditDataplaneTransferModel { state: None, flow_control: Some(flow_control) };

        let updated_process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .put_dataplane_transfers(&urn, &edit_model)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;
        let enriched = self.enrich_process(updated_process).await?;

        // Update cache
        let _ = self.cache.set_single(&urn, &enriched).await;

        Ok(enriched)
    }

    async fn delete_dataplane_transfer(&self, id: Uuid) -> anyhow::Result<()> {
        let urn_str = format!("urn:uuid:{}", id);
        let urn = Urn::from_str(&urn_str)
            .map_err(|e| CommonErrors::format_new(BadFormat::Unknown, &e.to_string()))?;

        self.data_plane_repo
            .get_dataplane_transfers_repo()
            .delete_dataplane_transfers(&urn)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        // Remove from cache
        let _ = self.cache.delete_single(&urn).await;

        Ok(())
    }
}
