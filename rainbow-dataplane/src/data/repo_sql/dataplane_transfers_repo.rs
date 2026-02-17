use crate::data::entities::dataplane_transfers::{
    self, EditDataplaneTransferModel, NewDataplaneTransferModel,
};
use crate::data::repo_traits::dataplane_transfers_repo::{
    DataplaneTransfersRepo, DataplaneTransfersRepoErrors,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use urn::Urn;
use uuid::Uuid;

pub struct DataplaneTransfersRepoForSql {
    db: DatabaseConnection,
}

impl DataplaneTransfersRepoForSql {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl DataplaneTransfersRepo for DataplaneTransfersRepoForSql {
    async fn get_all_dataplane_transfers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<dataplane_transfers::Model>, DataplaneTransfersRepoErrors> {
        let mut query = dataplane_transfers::Entity::find();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(page) = page {
            query = query.offset((page - 1) * limit.unwrap_or(10));
        }

        query.all(&self.db).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(anyhow::anyhow!(e))
        })
    }

    async fn get_batch_dataplane_transfers(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<dataplane_transfers::Model>, DataplaneTransfersRepoErrors> {
        let uuids: Vec<Uuid> = ids
            .iter()
            .filter_map(|urn| {
                // Assuming URN format urn:uuid:<uuid> or similar that allows extracting UUID.
                // Or if Urn is just a wrapper around string, we might need a way to get UUID.
                // For now, let's assume we can parse UUID from URN string or if it's already UUID URN.
                // If Urn doesn't support direct UUID extraction, we might need to rely on string parsing.
                // This is a placeholder logic.
                let parts: Vec<&str> = urn.nid().split(':').collect();
                if let Some(uuid_str) = parts.last() {
                    Uuid::parse_str(uuid_str).ok()
                } else {
                    None
                }
            })
            .collect();

        dataplane_transfers::Entity::find()
            .filter(dataplane_transfers::Column::Id.is_in(uuids))
            .all(&self.db)
            .await
            .map_err(|e| {
                DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(anyhow::anyhow!(e))
            })
    }

    async fn get_dataplane_transfers_by_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Option<dataplane_transfers::Model>, DataplaneTransfersRepoErrors> {
        let uuid = Uuid::parse_str(process_id.nid())
            .map_err(|_| DataplaneTransfersRepoErrors::DataplaneTransferNotFound)?;

        dataplane_transfers::Entity::find_by_id(uuid).one(&self.db).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(anyhow::anyhow!(e))
        })
    }

    async fn get_by_transfer_process_id(
        &self,
        transfer_process_id: &str,
    ) -> anyhow::Result<Option<dataplane_transfers::Model>, DataplaneTransfersRepoErrors> {
        dataplane_transfers::Entity::find()
            .filter(dataplane_transfers::Column::TransferProcessId.eq(transfer_process_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(anyhow::anyhow!(e))
            })
    }

    async fn create_dataplane_transfers(
        &self,
        new_dataplane_transfer: &NewDataplaneTransferModel,
    ) -> anyhow::Result<dataplane_transfers::Model, DataplaneTransfersRepoErrors> {
        let active_model: dataplane_transfers::ActiveModel = new_dataplane_transfer.clone().into();
        active_model.insert(&self.db).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorCreatingDataplaneTransfer(anyhow::anyhow!(e))
        })
    }

    async fn put_dataplane_transfers(
        &self,
        process_id: &Urn,
        new_dataplane_transfer: &EditDataplaneTransferModel,
    ) -> anyhow::Result<dataplane_transfers::Model, DataplaneTransfersRepoErrors> {
        let uuid = Uuid::parse_str(process_id.nid())
            .map_err(|_| DataplaneTransfersRepoErrors::DataplaneTransferNotFound)?;

        let mut active_model: dataplane_transfers::ActiveModel =
            new_dataplane_transfer.clone().into();
        active_model.id = Set(uuid);

        active_model.update(&self.db).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorUpdatingDataplaneTransfer(anyhow::anyhow!(e))
        })
    }

    async fn delete_dataplane_transfers(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<(), DataplaneTransfersRepoErrors> {
        let uuid = Uuid::parse_str(process_id.nid())
            .map_err(|_| DataplaneTransfersRepoErrors::DataplaneTransferNotFound)?;

        dataplane_transfers::Entity::delete_by_id(uuid).exec(&self.db).await.map_err(|e| {
            DataplaneTransfersRepoErrors::ErrorDeletingDataplaneTransfer(anyhow::anyhow!(e))
        })?;
        Ok(())
    }
}
