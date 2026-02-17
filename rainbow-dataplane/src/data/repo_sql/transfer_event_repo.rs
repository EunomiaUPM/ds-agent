use crate::data::entities::transfer_event;
use crate::data::entities::transfer_event::NewTransferEvent;
use crate::data::repo_traits::transfer_event_repo::{TransferEventRepo, TransferEventRepoErrors};
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect,
};
use urn::{Urn, UrnBuilder};
use uuid::Uuid;

pub struct TransferEventRepoForSql {
    db_connection: DatabaseConnection,
}
impl TransferEventRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl TransferEventRepo for TransferEventRepoForSql {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<transfer_event::Model>, TransferEventRepoErrors> {
        let events = transfer_event::Entity::find()
            .limit(limit.unwrap_or(20))
            .offset(page.map(|p| p * limit.unwrap_or(20)).unwrap_or(0))
            .all(&self.db_connection)
            .await;
        match events {
            Ok(events) => Ok(events),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into())),
        }
    }

    async fn get_batch_transfer_events(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<transfer_event::Model>, TransferEventRepoErrors> {
        let uuids: Vec<Uuid> =
            ids.iter().map(|urn| Uuid::parse_str(urn.nss())).filter_map(Result::ok).collect();

        let events = transfer_event::Entity::find()
            .filter(transfer_event::Column::Id.is_in(uuids))
            .all(&self.db_connection)
            .await;

        match events {
            Ok(events) => Ok(events),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into())),
        }
    }

    async fn get_all_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<transfer_event::Model>, TransferEventRepoErrors> {
        let uuid_str = process_id.nss();
        let uuid_val = Uuid::parse_str(uuid_str)
            .map_err(|e| TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into()))?;

        let events = transfer_event::Entity::find()
            .filter(transfer_event::Column::TransferId.eq(uuid_val))
            .all(&self.db_connection)
            .await;

        match events {
            Ok(events) => Ok(events),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into())),
        }
    }

    async fn get_transfer_event_by_id(
        &self,
        transfer_event_urn: &Urn,
    ) -> anyhow::Result<Option<transfer_event::Model>, TransferEventRepoErrors> {
        let uuid = Uuid::parse_str(transfer_event_urn.nss())
            .map_err(|e| TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into()))?;

        let event = transfer_event::Entity::find_by_id(uuid).one(&self.db_connection).await;

        match event {
            Ok(event) => Ok(event),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into())),
        }
    }

    async fn create_transfer_event(
        &self,
        data_plane_process: &Urn, // This is transfer_id
        new_transfer_event: &NewTransferEvent,
    ) -> anyhow::Result<transfer_event::Model, TransferEventRepoErrors> {
        let uuid_str = data_plane_process.nss();
        let transfer_id = Uuid::parse_str(uuid_str)
            .map_err(|e| TransferEventRepoErrors::ErrorCreatingTransferEvent(e.into()))?;

        let model = transfer_event::ActiveModel {
            transfer_id: ActiveValue::Set(transfer_id),
            level: ActiveValue::Set(new_transfer_event.level.clone()),
            component: ActiveValue::Set(new_transfer_event.component.clone()),
            message: ActiveValue::Set(new_transfer_event.message.clone()),
            data: ActiveValue::Set(new_transfer_event.data.clone()),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        let event =
            transfer_event::Entity::insert(model).exec_with_returning(&self.db_connection).await;
        match event {
            Ok(event) => Ok(event),
            Err(e) => return Err(TransferEventRepoErrors::ErrorCreatingTransferEvent(e.into())),
        }
    }
}
