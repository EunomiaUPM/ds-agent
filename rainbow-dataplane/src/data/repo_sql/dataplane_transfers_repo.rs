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
use ymir::errors::{Outcome, RepoIntoErrors};

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
    ) -> Outcome<Vec<dataplane_transfers::Model>> {
        let transfers = dataplane_transfers::Entity::find()
            .limit(limit.unwrap_or(100))
            .offset(page.map(|p| p * limit.unwrap_or(100)).unwrap_or(0))
            .all(&self.db)
            .await;
        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(e.into()).into_errors()),
        }
    }

    async fn get_batch_dataplane_transfers(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<dataplane_transfers::Model>> {
        let ids: Vec<String> = ids.iter().map(|urn| urn.to_string()).collect();
        let transfers = dataplane_transfers::Entity::find()
            .filter(dataplane_transfers::Column::Id.is_in(ids))
            .all(&self.db)
            .await;
        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(e.into()).into_errors()),
        }
    }

    async fn get_dataplane_transfers_by_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>> {
        let process_id = process_id.to_string();
        let transfer = dataplane_transfers::Entity::find_by_id(process_id).one(&self.db).await;
        match transfer {
            Ok(transfer) => Ok(transfer),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(e.into()).into_errors()),
        }
    }

    async fn get_by_transfer_process_id(
        &self,
        transfer_process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>> {
        let transfer_process_id = transfer_process_id.to_string();
        let transfer = dataplane_transfers::Entity::find()
            .filter(dataplane_transfers::Column::TransferProcessId.eq(transfer_process_id))
            .one(&self.db)
            .await;
        match transfer {
            Ok(transfer) => Ok(transfer),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorFetchingDataplaneTransfer(e.into()).into_errors()),
        }
    }

    async fn create_dataplane_transfers(
        &self,
        new_dataplane_transfer: &NewDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model> {
        let active_model: dataplane_transfers::ActiveModel = new_dataplane_transfer.clone().into();
        let transfer = active_model.insert(&self.db).await;
        match transfer {
            Ok(transfer) => Ok(transfer),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorCreatingDataplaneTransfer(e.into()).into_errors()),
        }
    }

    async fn put_dataplane_transfers(
        &self,
        process_id: &Urn,
        new_dataplane_transfer: &EditDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model> {
        let process_id = process_id.to_string();
        let mut active_model: dataplane_transfers::ActiveModel =
            new_dataplane_transfer.clone().into();
        active_model.id = Set(process_id);

        let transfer = active_model.update(&self.db).await;
        match transfer {
            Ok(transfer) => Ok(transfer),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorUpdatingDataplaneTransfer(e.into()).into_errors()),
        }
    }

    async fn delete_dataplane_transfers(
        &self,
        process_id: &Urn,
    ) -> Outcome<()> {
        let process_id = process_id.to_string();
        let transfer = dataplane_transfers::Entity::delete_by_id(process_id).exec(&self.db).await;
        match transfer {
            Ok(_) => Ok(()),
            Err(e) => Err(DataplaneTransfersRepoErrors::ErrorDeletingDataplaneTransfer(e.into()).into_errors()),
        }
    }
}
