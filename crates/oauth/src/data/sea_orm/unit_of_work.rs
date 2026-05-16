use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use tokio::sync::Mutex;
use ymir::errors::{Errors, Outcome};

use crate::data::unit_of_work::UnitOfWork;

pub(crate) struct SeaOrmUow {
    txn: Mutex<Option<DatabaseTransaction>>,
}

impl SeaOrmUow {
    pub async fn begin(db: &DatabaseConnection) -> Outcome<Self> {
        let txn = db
            .begin()
            .await
            .map_err(|e| Errors::crazy("failed to begin transaction", Some(Box::new(e))))?;
        Ok(Self {
            txn: Mutex::new(Some(txn)),
        })
    }
}

#[async_trait::async_trait]
impl UnitOfWork for SeaOrmUow {
    async fn commit(&self) -> Outcome<()> {
        if let Some(txn) = self.txn.lock().await.take() {
            txn.commit()
                .await
                .map_err(|e| Errors::crazy("transaction commit failed", Some(Box::new(e))))?;
        }
        Ok(())
    }

    async fn rollback(&self) -> Outcome<()> {
        if let Some(txn) = self.txn.lock().await.take() {
            txn.rollback()
                .await
                .map_err(|e| Errors::crazy("transaction rollback failed", Some(Box::new(e))))?;
        }
        Ok(())
    }
}
