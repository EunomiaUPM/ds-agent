use std::sync::Arc;

use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;

pub(crate) trait DataFactory: Send + Sync {
    fn transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait>;
    fn transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait>;
    fn transfer_identifier_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait>;
}
