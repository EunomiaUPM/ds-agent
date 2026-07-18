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

use axum::Router;
use common::auth::middleware::OauthTokenValidator;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use std::sync::Arc;
use tonic::service::RoutesBuilder;
use ymir::config::traits::ApiConfigTrait;

use crate::grpc::api::transfer_messages::transfer_messages_ref_server::TransferMessagesRefServer;
use crate::grpc::api::transfer_processes::transfer_processes_ref_server::TransferProcessesRefServer;
use crate::grpc::transfer_messages::TransferMessagesGrpc;
use crate::grpc::transfer_process::TransferProcessGrpc;
use crate::http::build_router;
use crate::http::transfer_message_router::TransferMessageRouter;
use crate::http::transfer_process_router::TransferProcessRouter;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;

pub struct ApiModule {
    base_path: String,
    process: Arc<TransferProcessService>,
    message: Arc<TransferMessageService>,
    oauth_validator: Arc<dyn OauthTokenValidator>,
}

impl ApiModule {
    pub fn new(
        config: Arc<TransferConfig>,
        process: Arc<TransferProcessService>,
        message: Arc<TransferMessageService>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            base_path: format!("{}/transfer-agent-ref", config.common().get_api_version()),
            process,
            message,
            oauth_validator: validator,
        }
    }
}

impl ServiceModuleTrait for ApiModule {
    fn name(&self) -> &'static str {
        "transfer-api"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        crate::data::sea_orm::migrations::get_migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let process_router = Router::new().nest(
            "/transfer-processes",
            TransferProcessRouter::new(self.process.clone()).router(),
        );
        let message_router = Router::new().nest(
            "/transfer-messages",
            TransferMessageRouter::new(self.message.clone()).router(),
        );
        Some((
            self.base_path.clone(),
            build_router(self.oauth_validator.clone(), process_router, message_router),
        ))
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        let process_handler =
            TransferProcessGrpc::new(self.process.clone(), self.oauth_validator.clone());
        let message_handler =
            TransferMessagesGrpc::new(self.message.clone(), self.oauth_validator.clone());
        routes
            .add_service(TransferProcessesRefServer::new(process_handler))
            .add_service(TransferMessagesRefServer::new(message_handler));
    }
}
