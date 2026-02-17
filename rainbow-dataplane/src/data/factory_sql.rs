use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::repo_sql::dataplane_fields_repo::DataplaneFieldRepoForSql;
use crate::data::repo_sql::dataplane_transfer_logs_repo::DataplaneTransferLogsRepoForSql;
use crate::data::repo_sql::dataplane_transfers_repo::DataplaneTransfersRepoForSql;
use crate::data::repo_sql::transfer_event_repo::TransferEventRepoForSql;
use crate::data::repo_traits::dataplane_fields_repo::DataplaneFieldRepoTrait;
use crate::data::repo_traits::dataplane_transfer_logs_repo::DataplaneTransferLogsRepo;
use crate::data::repo_traits::dataplane_transfers_repo::DataplaneTransfersRepo;
use crate::data::repo_traits::transfer_event_repo::TransferEventRepo;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct DataplaneRepoForSql {
    dataplane_transfers_repo: Arc<dyn DataplaneTransfersRepo>,
    dataplane_fields_repo: Arc<dyn DataplaneFieldRepoTrait>,
    dataplane_transfer_logs_repo: Arc<dyn DataplaneTransferLogsRepo>,
    transfer_events_repo: Arc<dyn TransferEventRepo>,
}

impl DataplaneRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            dataplane_transfers_repo: Arc::new(DataplaneTransfersRepoForSql::new(
                db_connection.clone(),
            )),
            dataplane_fields_repo: Arc::new(DataplaneFieldRepoForSql::new(db_connection.clone())),
            dataplane_transfer_logs_repo: Arc::new(DataplaneTransferLogsRepoForSql::new(
                db_connection.clone(),
            )),
            transfer_events_repo: Arc::new(TransferEventRepoForSql::new(db_connection.clone())),
        }
    }
}

impl DataplaneRepoTrait for DataplaneRepoForSql {
    fn get_dataplane_transfers_repo(&self) -> Arc<dyn DataplaneTransfersRepo> {
        self.dataplane_transfers_repo.clone()
    }

    fn get_dataplane_fields_repo(&self) -> Arc<dyn DataplaneFieldRepoTrait> {
        self.dataplane_fields_repo.clone()
    }

    fn get_dataplane_transfer_logs_repo(&self) -> Arc<dyn DataplaneTransferLogsRepo> {
        self.dataplane_transfer_logs_repo.clone()
    }

    fn get_transfer_events_repo(&self) -> Arc<dyn TransferEventRepo> {
        self.transfer_events_repo.clone()
    }
}
