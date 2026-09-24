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

//! OAuth as a composable module: the `/oauth` endpoints (token, users, clients, pats).

use std::sync::Arc;

use axum::Router;
use common::auth::OauthTokenValidator;
use common::config::services::CommonConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;

use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::http::clients_router::ClientsRouter;
use crate::http::pats_router::PatsRouter;
use crate::http::token_router::TokenRouter;
use crate::http::users_router::UsersRouter;
use crate::setup::context::AppContext;

pub struct OAuthModule {
    ctx: AppContext,
}

impl OAuthModule {
    pub fn compose(
        common: &CommonConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        Self {
            ctx: AppContext::build(common.clone().into(), root.db.clone(), event_bus),
        }
    }

    /// Process token validator; matches `common::module_loader::root_context::ValidatorFactory`.
    pub fn validator(
        common: &CommonConfig,
        db: DatabaseConnection,
    ) -> Arc<dyn OauthTokenValidator> {
        AppContext::token_service(&common.clone().into(), &SeaOrmDataFactory::new(db))
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::get_oauth_migrations()
    }
}

impl ServiceModuleTrait for OAuthModule {
    fn name(&self) -> &'static str {
        "oauth"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let ctx = &self.ctx;
        let validator: Arc<dyn OauthTokenValidator> = ctx.token_svc.clone();
        let token_router = TokenRouter::new(
            ctx.token_svc.clone(),
            ctx.user_svc.clone(),
            ctx.config.issuer.clone(),
        )
        .router();
        let protected = Router::new()
            .nest("/users", UsersRouter::new(ctx.user_svc.clone()).router())
            .nest(
                "/clients",
                ClientsRouter::new(ctx.client_svc.clone()).router(),
            )
            .nest("/pats", PatsRouter::new(ctx.pat_svc.clone()).router())
            .route_layer(axum::middleware::from_fn_with_state(
                validator,
                common::auth::http::AuthHttpMiddleware::run,
            ));
        Some((
            "/oauth".to_string(),
            Router::new().merge(token_router).merge(protected),
        ))
    }
}
