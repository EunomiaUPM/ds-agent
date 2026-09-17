use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use common::module_loader::service_module::ServiceModuleTrait;
use crate::config::OAuthConfig;
use crate::setup::composition::OAuthSetup;

/// OAuth as a composable service module: `/oauth` endpoints (token, refresh,
/// revoke, introspect, users, clients). Construct it with config and DB connection.
pub struct OAuthModule {
    config: OAuthConfig,
    db: DatabaseConnection,
    event_bus: Option<events::EventBus>,
}

impl OAuthModule {
    pub fn new(config: OAuthConfig, db: DatabaseConnection) -> Self {
        Self {
            config,
            db,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

impl ServiceModuleTrait for OAuthModule {
    fn name(&self) -> &'static str {
        "oauth"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        crate::get_oauth_migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let router = OAuthSetup::new().build_router_with_bus(
            self.config.clone(),
            self.db.clone(),
            self.event_bus.clone(),
        );
        Some(("/oauth".to_string(), router))
    }
}