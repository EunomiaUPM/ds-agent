use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use tokio::sync::Mutex;
use ymir::errors::{Errors, Outcome};

use crate::data::uow::UnitOfWorkTrait;

pub(crate) struct SeaOrmUow {
    txn: Mutex<Option<DatabaseTransaction>>,
}

impl SeaOrmUow {
    pub async fn begin(db: &DatabaseConnection) -> Outcome<Self> {
        let txn = db
            .begin()
            .await
            .map_err(|e| Errors::crazy("failed to begin transaction", Some(Box::new(e))))?;
        Ok(Self { txn: Mutex::new(Some(txn)) })
    }
}

impl UnitOfWorkTrait for SeaOrmUow {
    async fn commit(&self) -> Outcome<()> {
        self.txn
            .lock()
            .await
            .take()
            .ok_or_else(|| Errors::crazy("transaction already consumed", None))?
            .commit()
            .await
            .map_err(|e| Errors::crazy("transaction commit failed", Some(Box::new(e))))
    }

    async fn rollback(&self) -> Outcome<()> {
        self.txn
            .lock()
            .await
            .take()
            .ok_or_else(|| Errors::crazy("transaction already consumed", None))?
            .rollback()
            .await
            .map_err(|e| Errors::crazy("transaction rollback failed", Some(Box::new(e))))
    }
}
