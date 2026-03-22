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

use crate::data::repo_traits::transfer_message_repo::TransferMessageRepoTrait;
use crate::data::repo_traits::transfer_process_identifier_repo::TransferIdentifierRepoTrait;
use crate::data::repo_traits::transfer_process_repo::TransferProcessRepoTrait;
use std::sync::Arc;

#[mockall::automock]
pub trait TransferAgentRepoTrait: Send + Sync + 'static {
    fn get_transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait>;
    fn get_transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait>;
    fn get_transfer_process_identifiers_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait>;
}
