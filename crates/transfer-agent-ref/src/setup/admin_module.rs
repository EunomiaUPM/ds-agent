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
use crate::http::build_router;
use crate::http::transfer_message_router::TransferMessageRouter;
use crate::http::transfer_process_router::TransferProcessRouter;
use crate::setup::context::AppContext;

/// This agent's own transfer-processes / transfer-messages surface, behind OAuth
/// token validation. Split out as its own module so it composes as a peer of the
/// hosted OAuth and DSP modules rather than as special-cased glue.
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
        "transfer-agent-native"
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
        let router = build_router(
            self.ctx.oauth_validator.clone(),
            process_router,
            message_router,
        );
        Some((self.base_path(), router))
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
