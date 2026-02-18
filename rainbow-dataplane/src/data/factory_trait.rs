use crate::data::repo_traits::dataplane_fields_repo::DataplaneFieldRepoTrait;
use crate::data::repo_traits::dataplane_transfer_logs_repo::DataplaneTransferLogsRepo;
use crate::data::repo_traits::dataplane_transfers_repo::DataplaneTransfersRepo;
use crate::data::repo_traits::transfer_event_repo::TransferEventRepo;
use std::sync::Arc;

pub trait DataplaneRepoTrait: Send + Sync + 'static {
    fn get_dataplane_transfers_repo(&self) -> Arc<dyn DataplaneTransfersRepo>;
    fn get_dataplane_fields_repo(&self) -> Arc<dyn DataplaneFieldRepoTrait>;
    fn get_dataplane_transfer_logs_repo(&self) -> Arc<dyn DataplaneTransferLogsRepo>;
    fn get_transfer_events_repo(&self) -> Arc<dyn TransferEventRepo>;
}
