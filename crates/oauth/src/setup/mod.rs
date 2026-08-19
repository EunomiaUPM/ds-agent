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
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;

use crate::config::OAuthConfig;
use crate::data::factory::OAuthDataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::http::token_router::TokenRouter;
use crate::http::users_router::UsersRouter;
use crate::services::token_service::service::TokenService;
use crate::services::token_service::{OauthTokenValidator, TokenServiceTrait};
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::service::UserService;

/// OAuth as a composable service module: `/oauth` endpoints (login / token /
/// refresh / users) plus the users tables. Construct it with the config and
/// DB connection it should serve from.
pub struct OAuthModule {
    config: OAuthConfig,
    db: DatabaseConnection,
}

impl OAuthModule {
    pub fn new(config: OAuthConfig, db: DatabaseConnection) -> Self {
        Self { config, db }
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
        let router = OAuthSetup::new().build_router(self.config.clone(), self.db.clone());
        Some(("/oauth".to_string(), router))
    }
}

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
            config,
        ))
    }

    /// Builds the full OAuth router (token, users)
    /// Mount this under an appropriate prefix (e.g. `/oauth`) in the host service.
    pub fn build_router(&self, config: OAuthConfig, db: DatabaseConnection) -> Router {
        let factory = SeaOrmDataFactory::new(db.clone());
        let token_svc: Arc<dyn TokenServiceTrait> =
            self.build_token_service(config.clone(), db.clone());
        let user_svc: Arc<dyn UserServiceTrait> =
            Arc::new(UserService::new(factory.user_repository()));
        let issuer = config.issuer.clone();
        let token_router = TokenRouter::new(token_svc.clone(), user_svc.clone(), issuer).router();
        let users_router = UsersRouter::new(token_svc, user_svc).router();

        Router::new()
            .merge(token_router)
            .nest("/users", users_router)
    }
}
