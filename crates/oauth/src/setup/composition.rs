use crate::config::OAuthConfig;
use crate::data::factory::OAuthDataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::http::clients_router::ClientsRouter;
use crate::http::pats_router::PatsRouter;
use crate::http::token_router::TokenRouter;
use crate::http::users_router::UsersRouter;
use crate::services::client_service::ClientServiceTrait;
use crate::services::client_service::service::ClientService;
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::service::PatService;
use crate::services::token_service::TokenServiceTrait;
use crate::services::token_service::service::TokenService;
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::service::UserService;
use axum::Router;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Default)]
pub struct OAuthSetup {}

impl OAuthSetup {
    pub fn new() -> Self {
        OAuthSetup {}
    }

    /// Token services for OAuth token validation
    pub fn build_token_service(
        &self,
        config: OAuthConfig,
        db: DatabaseConnection,
    ) -> Arc<dyn TokenServiceTrait> {
        let factory = SeaOrmDataFactory::new(db);
        Arc::new(TokenService::new(
            factory.user_repository(),
            factory.token_repository(),
            factory.client_repository(),
            factory.auth_code_repository(),
            factory.pat_repository(),
            config,
        ))
    }

    /// Builds the full OAuth router (token, users, clients, pats)
    /// Mount this under an appropriate prefix (e.g. `/oauth`) in the host service.
    pub fn build_router(&self, config: OAuthConfig, db: DatabaseConnection) -> Router {
        self.build_router_with_bus(config, db, None)
    }

    pub fn build_router_with_bus(
        &self,
        config: OAuthConfig,
        db: DatabaseConnection,
        event_bus: Option<events::EventBus>,
    ) -> Router {
        let factory = SeaOrmDataFactory::new(db.clone());
        let token_service = Arc::new(TokenService::new(
            factory.user_repository(),
            factory.token_repository(),
            factory.client_repository(),
            factory.auth_code_repository(),
            factory.pat_repository(),
            config.clone(),
        ));
        let token_svc: Arc<dyn TokenServiceTrait> = token_service.clone();
        let validator: Arc<dyn common::auth::OauthTokenValidator> = token_service;
        let user_svc: Arc<dyn UserServiceTrait> =
            Arc::new(UserService::new(factory.user_repository()).with_event_bus(event_bus.clone()));
        let client_svc: Arc<dyn ClientServiceTrait> = Arc::new(
            ClientService::new(factory.client_repository()).with_event_bus(event_bus.clone()),
        );
        let pat_svc: Arc<dyn PatServiceTrait> =
            Arc::new(PatService::new(factory.pat_repository()).with_event_bus(event_bus));
        let issuer = config.issuer.clone();
        let token_router = TokenRouter::new(token_svc.clone(), user_svc.clone(), issuer).router();
        let users_router = UsersRouter::new(user_svc).router();
        let clients_router = ClientsRouter::new(client_svc).router();
        let pats_router = PatsRouter::new(pat_svc).router();

        let protected = Router::new()
            .nest("/users", users_router)
            .nest("/clients", clients_router)
            .nest("/pats", pats_router)
            .route_layer(axum::middleware::from_fn_with_state(
                validator,
                common::auth::http::AuthHttpMiddleware::run,
            ));

        Router::new().merge(token_router).merge(protected)
    }
}
