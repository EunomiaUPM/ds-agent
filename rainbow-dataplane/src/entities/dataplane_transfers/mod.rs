pub(crate) mod dataplane_transfers_entity;

use crate::data::entities::dataplane_transfers;
pub use crate::data::entities::dataplane_transfers::{
    InteractionMode, NewDataplaneTransfer, TransferRole, TransferState,
};
use crate::data::entities::{dataplane_field, dataplane_transfer_logs, transfer_event};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataplaneTransferDto {
    #[serde(flatten)]
    pub inner: dataplane_transfers::Model,
    pub fields: HashMap<String, String>,
    pub logs: Vec<dataplane_transfer_logs::Model>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDataplaneTransferDto {
    pub id: Option<Urn>,
    pub transfer_process_id: String,
    pub role: TransferRole,
    pub interaction_mode: InteractionMode,
    pub state: TransferState,
    pub connector_instance_id: Option<Urn>,
    pub ingress_config: Value,
    pub egress_config: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDataplaneTransferDto {
    pub state: Option<TransferState>,
    pub connector_instance_id: Option<Urn>,
    pub ingress_config: Option<Value>,
    pub egress_config: Option<Value>,
    pub flow_control: Option<Value>,
    pub fields: Option<HashMap<String, String>>,
}

impl From<NewDataplaneTransferDto> for NewDataplaneTransfer {
    fn from(value: NewDataplaneTransferDto) -> Self {
        Self {
            id: value.id,
            transfer_process_id: value.transfer_process_id,
            role: value.role,
            interaction_mode: value.interaction_mode,
            state: value.state,
            connector_instance_id: value.connector_instance_id,
            ingress_config: value.ingress_config,
            egress_config: value.egress_config,
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DataplaneTransfersEntitiesTrait: Send + Sync + 'static {
    async fn get_all_dataplane_transfers(&self) -> Outcome<Vec<DataplaneTransferDto>>;

    async fn get_dataplane_transfer_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<DataplaneTransferDto>>;

    async fn get_dataplane_transfer_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Option<DataplaneTransferDto>>;

    async fn get_batch_dataplane_transfers(
        &self,
        transfer_ids: &Vec<Urn>,
    ) -> Outcome<Vec<DataplaneTransferDto>>;

    async fn create_dataplane_transfer(
        &self,
        new_data_plane_process: &NewDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto>;

    async fn put_dataplane_transfer_by_id(
        &self,
        id: &Urn,
        edit_dataplane_transfer: &EditDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto>;

    async fn delete_dataplane_transfer(&self, id: &Urn) -> Outcome<()>;
}
