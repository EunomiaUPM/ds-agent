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

use crate::TransferDummyTrait;
use std::sync::Arc;
use ymir::errors::Outcome;

#[allow(unused)]
pub struct TransferSharedServices {
    // Repo
    pub transfer_process_repo: Arc<dyn TransferDummyTrait>,
    pub transfer_identifiers_repo: Arc<dyn TransferDummyTrait>,
    pub transfer_message_repo: Arc<dyn TransferDummyTrait>,
    pub auth_facade: Arc<dyn TransferDummyTrait>,
    pub dataplane_facade: Arc<dyn TransferDummyTrait>,
    pub notifier: Arc<dyn TransferDummyTrait>,
    pub agreement_facade: Arc<dyn TransferDummyTrait>,
    pub data_service_facade: Arc<dyn TransferDummyTrait>,
}

#[async_trait::async_trait]
#[allow(unused)]
pub trait ProtocolPluginTrait {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn short_name(&self) -> &'static str;
    async fn build_router(&self) -> Outcome<axum::Router>;
    fn build_grpc_router(&self) -> Outcome<Option<axum::Router>>;
}
