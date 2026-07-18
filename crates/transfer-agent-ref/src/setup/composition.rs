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

//! Composition root: declares the agent's modules and derives everything the
//! agent exposes — HTTP, gRPC and DB migrations — from that single tree.

use axum::Router;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::service_composer::ServiceComposer;
use common::module_loader::service_module::ServiceModuleTrait;
use oauth::setup::OAuthModule;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::Arc;
use tonic::service::{Routes, RoutesBuilder};
use ymir::config::traits::ApiConfigTrait;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::grpc::api::transfer_messages::transfer_messages_ref_server::TransferMessagesRefServer;
use crate::grpc::api::transfer_processes::transfer_processes_ref_server::TransferProcessesRefServer;
use crate::grpc::transfer_messages::TransferMessagesGrpc;
use crate::grpc::transfer_process::TransferProcessGrpc;
use crate::http::build_router;
use crate::http::transfer_message_router::TransferMessageRouter;
use crate::http::transfer_process_router::TransferProcessRouter;
use crate::protocols::dsp::setup::DspModule;
use crate::setup::context::ModuleCtx;

// Module tree ────────────────────────────────────────────────────────────────

/// The agent's full module set, defined once and shared by the workers and
/// the migrator.
fn agent_modules() -> ModuleGroup<ModuleCtx> {
    ModuleGroup::new("transfer-agent-ref")
        .register(OAuthModule)
        .register(ApiModule)
        .register(DspModule)
}

struct ApiModule;

impl ServiceModuleTrait<ModuleCtx> for ApiModule {
    fn name(&self) -> &'static str {
        "transfer-api"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        crate::data::sea_orm::migrations::get_migrations()
    }

    fn http(&self, ctx: &ModuleCtx) -> Option<(String, Router)> {
        let process_router = Router::new().nest(
            "/transfer-processes",
            TransferProcessRouter::new(ctx.services.process.clone()).router(),
        );
        let message_router = Router::new().nest(
            "/transfer-messages",
            TransferMessageRouter::new(ctx.services.message.clone()).router(),
        );
        let base = format!(
            "{}/transfer-agent-ref",
            ctx.config.common().get_api_version()
        );
        Some((
            base,
            build_router(
                ctx.services.validator.clone(),
                process_router,
                message_router,
            ),
        ))
    }

    fn grpc(&self, ctx: &ModuleCtx, routes: &mut RoutesBuilder) {
        let process_handler =
            TransferProcessGrpc::new(ctx.services.process.clone(), ctx.services.validator.clone());
        let message_handler =
            TransferMessagesGrpc::new(ctx.services.message.clone(), ctx.services.validator.clone());
        routes
            .add_service(TransferProcessesRefServer::new(process_handler))
            .add_service(TransferMessagesRefServer::new(message_handler));
    }
}

// Composition ────────────────────────────────────────────────────────────────

pub(crate) struct TransferAgentRefCompositionRoot {
    pub http_router: Router,
    pub grpc_routes: Routes,
}

impl TransferAgentRefCompositionRoot {
    pub async fn compose(config: &TransferConfig, vault: &VaultService) -> Outcome<Self> {
        let ctx = ModuleCtx::build(config, vault).await?;
        let composer = ServiceComposer::new(ctx).register(agent_modules());

        Ok(Self {
            http_router: composer.http_router(),
            grpc_routes: composer.grpc_routes(),
        })
    }
}

pub struct TransferAgentRefMigration;

impl MigratorTrait for TransferAgentRefMigration {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        agent_modules().migrations()
    }
}

impl TransferAgentRefMigration {
    pub async fn run(config: &TransferConfig, vault: Arc<VaultService>) -> Outcome<()> {
        let db_connection = vault.get_db_connection(config.common()).await?;
        Self::refresh(&db_connection)
            .await
            .map_err(|e| Errors::crazy("Not able to run migration", Some(Box::new(e))))?;
        Ok(())
    }
}
