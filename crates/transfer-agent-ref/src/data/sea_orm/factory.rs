use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::data::factory::DataFactory;
use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;
use crate::data::sea_orm::repos::transfer_identifier::SeaOrmTransferIdentifierRepo;
use crate::data::sea_orm::repos::transfer_message::SeaOrmTransferMessageRepo;
use crate::data::sea_orm::repos::transfer_process::SeaOrmTransferProcessRepo;

pub(crate) struct SeaOrmDataFactory {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmDataFactory {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }
}

impl DataFactory for SeaOrmDataFactory {
    fn transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait> {
        Arc::new(SeaOrmTransferProcessRepo::new(self.db.clone()))
    }

    fn transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait> {
        Arc::new(SeaOrmTransferMessageRepo::new(self.db.clone()))
    }

    fn transfer_identifier_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait> {
        Arc::new(SeaOrmTransferIdentifierRepo::new(self.db.clone()))
    }
}
