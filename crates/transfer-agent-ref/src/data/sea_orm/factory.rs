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
