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

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::repo::dataplane_field::DataplaneFieldRepoTrait;
use crate::data::repo::dataplane_transfer::DataplaneTransfersRepo;
use crate::data::repo::dataplane_transfer_log::DataplaneTransferLogsRepo;
use crate::data::repo::transfer_event::TransferEventRepo;
use crate::data::sea_orm::repos::dataplane_field::DataplaneFieldRepoForSql;
use crate::data::sea_orm::repos::dataplane_transfer::DataplaneTransfersRepoForSql;
use crate::data::sea_orm::repos::dataplane_transfer_log::DataplaneTransferLogsRepoForSql;
use crate::data::sea_orm::repos::transfer_event::TransferEventRepoForSql;

pub struct SeaOrmDataFactory {
    dataplane_transfers_repo: Arc<dyn DataplaneTransfersRepo>,
    dataplane_fields_repo: Arc<dyn DataplaneFieldRepoTrait>,
    dataplane_transfer_logs_repo: Arc<dyn DataplaneTransferLogsRepo>,
    transfer_events_repo: Arc<dyn TransferEventRepo>,
}

impl SeaOrmDataFactory {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        let db = Arc::new(db_connection);
        Self {
            dataplane_transfers_repo: Arc::new(DataplaneTransfersRepoForSql::new(db.clone())),
            dataplane_fields_repo: Arc::new(DataplaneFieldRepoForSql::new(db.clone())),
            dataplane_transfer_logs_repo: Arc::new(DataplaneTransferLogsRepoForSql::new(
                db.clone(),
            )),
            transfer_events_repo: Arc::new(TransferEventRepoForSql::new(db)),
        }
    }

    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self::new(db_connection)
    }
}

impl DataplaneRepoTrait for SeaOrmDataFactory {
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

pub type DataplaneRepoForSql = SeaOrmDataFactory;
