/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

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
