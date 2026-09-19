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

use axum::Router;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::config::traits::ApiConfigTrait;

use crate::SERVICE_NAME;
use crate::grpc::api::transfer_messages::transfer_messages_ref_server::TransferMessagesRefServer;
use crate::grpc::api::transfer_processes::transfer_processes_ref_server::TransferProcessesRefServer;
use crate::grpc::transfer_messages::TransferMessagesGrpc;
use crate::grpc::transfer_process::TransferProcessGrpc;
use crate::http::TransferHttpRouter;
use crate::http::transfer_message_router::TransferMessageRouter;
use crate::http::transfer_process_router::TransferProcessRouter;
use crate::setup::context::AppContext;

pub struct TransferAdminModule {
    ctx: Arc<AppContext>,
}

impl TransferAdminModule {
    pub(crate) fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Mount prefix of this agent's own API, e.g. `/api/v1/transfer-agent-ref`.
    fn base_path(&self) -> String {
        format!(
            "{}/{}",
            self.ctx.config.common().get_api_version(),
            SERVICE_NAME
        )
    }
}

impl ServiceModuleTrait for TransferAdminModule {
    fn name(&self) -> &'static str {
        "transfer-agent"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        crate::data::sea_orm::migrations::get_migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let process_router = Router::new().nest(
            "/transfer-processes",
            TransferProcessRouter::new(self.ctx.transfer_process_svc.clone()).router(),
        );
        let message_router = Router::new().nest(
            "/transfer-messages",
            TransferMessageRouter::new(self.ctx.transfer_message_svc.clone()).router(),
        );
        let router = TransferHttpRouter::build(
            process_router,
            message_router,
            self.ctx.oauth_validator.clone(),
        );
        let api_version = self.ctx.config.common().get_api_version();
        let combined = Router::new()
            .nest(&format!("{}/transfer-agent", api_version), router.clone())
            .nest(&self.base_path(), router);
        Some((String::new(), combined))
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        routes
            .add_service(TransferProcessesRefServer::new(TransferProcessGrpc::new(
                self.ctx.transfer_process_svc.clone(),
                self.ctx.oauth_validator.clone(),
            )))
            .add_service(TransferMessagesRefServer::new(TransferMessagesGrpc::new(
                self.ctx.transfer_message_svc.clone(),
                self.ctx.oauth_validator.clone(),
            )));
    }
}
