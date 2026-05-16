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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::data::factory::DataFactory;
use crate::data::in_memory::repos::{
    InMemoryTransferIdentifierRepo, InMemoryTransferMessageRepo, InMemoryTransferProcessRepo,
};
use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::data::repo::transfer_process::TransferProcessRepoTrait;
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;

pub(crate) struct InMemoryDataFactory {
    processes: Arc<Mutex<HashMap<String, crate::entities::transfer_process::TransferProcess>>>,
    messages: Arc<Mutex<HashMap<String, crate::entities::transfer_message::TransferMessage>>>,
    identifiers: Arc<
        Mutex<
            HashMap<
                (String, String),
                crate::entities::transfer_process_identifier::TransferProcessIdentifier,
            >,
        >,
    >,
}

impl InMemoryDataFactory {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            identifiers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl DataFactory for InMemoryDataFactory {
    fn transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait> {
        Arc::new(InMemoryTransferProcessRepo::new(
            self.processes.clone(),
            self.identifiers.clone(),
        ))
    }

    fn transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait> {
        Arc::new(InMemoryTransferMessageRepo::new(self.messages.clone()))
    }

    fn transfer_identifier_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait> {
        Arc::new(InMemoryTransferIdentifierRepo::new(
            self.identifiers.clone(),
        ))
    }
}
