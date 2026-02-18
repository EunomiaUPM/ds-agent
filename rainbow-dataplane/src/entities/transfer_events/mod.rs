pub(crate) mod transfer_event_entity;

use crate::data::entities::transfer_event;
use crate::data::entities::transfer_event::{LogLevel, NewTransferEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEventDto {
    #[serde(flatten)]
    pub inner: transfer_event::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewTransferEventDto {
    pub transfer_id: Urn,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub data: Option<Value>,
}

impl From<NewTransferEventDto> for NewTransferEvent {
    fn from(value: NewTransferEventDto) -> Self {
        Self {
            transfer_id: value.transfer_id.to_string(),
            level: value.level,
            component: value.component,
            message: value.message,
            data: value.data,
        }
    }
}

#[async_trait::async_trait]
pub trait TransferEventEntitiesTrait: Send + Sync + 'static {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<TransferEventDto>>;

    async fn get_batch_transfer_events(
        &self,
        ids: Vec<Urn>,
    ) -> anyhow::Result<Vec<TransferEventDto>>;

    async fn get_transfer_event_by_id(&self, id: &Urn) -> anyhow::Result<Option<TransferEventDto>>;

    async fn get_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<TransferEventDto>>;

    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEventDto,
    ) -> anyhow::Result<TransferEventDto>;
}
