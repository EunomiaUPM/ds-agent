use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::config::OAuthConfig;
use crate::services::token_service::{TokenServiceTrait, TokenValidator};

pub struct TokenServiceSetup {}

impl TokenServiceSetup {
    pub fn new() -> Self {
        TokenServiceSetup {}
    }

    /// Validate-only service backed by in-memory stubs.
    /// Only `validate_token` is functional; all other methods will fail because
    /// the backing repos are empty.
    pub fn build_validator(&self, jwt_secret: &str) -> Arc<dyn TokenValidator> {
        use crate::data::in_memory::repos::{
            InMemoryRefreshTokenRepository, InMemoryUserRepository,
        };
        use crate::services::token_service::service::TokenService;
        Arc::new(TokenService::new(
            Arc::new(InMemoryUserRepository::new()),
            Arc::new(InMemoryRefreshTokenRepository::new()),
            OAuthConfig::new(jwt_secret, "", ""),
        ))
    }

    /// Full token service backed by a real database.
    pub fn build_full(
        &self,
        config: OAuthConfig,
        db: DatabaseConnection,
    ) -> Arc<dyn TokenServiceTrait> {
        use crate::data::factory::OAuthDataFactory;
        use crate::data::sea_orm::factory::SeaOrmDataFactory;
        use crate::services::token_service::service::TokenService;
        let factory = SeaOrmDataFactory::new(db);
        Arc::new(TokenService::new(
            factory.user_repository(),
            factory.refresh_token_repository(),
            config,
        ))
    }
}
