/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::factory_trait::TransferAgentRepoTrait;
use crate::data::repo_traits::transfer_message_repo::TransferMessageRepoTrait;
use crate::data::repo_traits::transfer_process_identifier_repo::TransferIdentifierRepoTrait;
use crate::data::repo_traits::transfer_process_repo::TransferProcessRepoTrait;
use crate::data::repos_sql::transfer_message_repo::TransferMessageRepoForSql;
use crate::data::repos_sql::transfer_process_identifier_repo::TransferIdentifierRepoForSql;
use crate::data::repos_sql::transfer_process_repo::TransferProcessRepoForSql;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct TransferAgentRepoForSql {
    transfer_process_repo: Arc<dyn TransferProcessRepoTrait>,
    transfer_process_identifier_repo: Arc<dyn TransferIdentifierRepoTrait>,
    transfer_message_repo: Arc<dyn TransferMessageRepoTrait>,
}

impl TransferAgentRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            transfer_process_repo: Arc::new(TransferProcessRepoForSql::new(db_connection.clone())),
            transfer_process_identifier_repo: Arc::new(TransferIdentifierRepoForSql::new(
                db_connection.clone(),
            )),
            transfer_message_repo: Arc::new(TransferMessageRepoForSql::new(db_connection.clone())),
        }
    }
}

impl TransferAgentRepoTrait for TransferAgentRepoForSql {
    fn get_transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait> {
        self.transfer_process_repo.clone()
    }
    fn get_transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait> {
        self.transfer_message_repo.clone()
    }
    fn get_transfer_process_identifiers_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait> {
        self.transfer_process_identifier_repo.clone()
    }
}
