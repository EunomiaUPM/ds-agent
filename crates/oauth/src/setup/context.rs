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

//! Wires the OAuth data layer and services once, for the token validator and every router.

use std::sync::Arc;

use crate::config::OAuthConfig;
use crate::data::factory::OAuthDataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::client_service::service::ClientService;
use crate::services::pat_service::service::PatService;
use crate::services::token_service::service::TokenService;
use crate::services::user_service::service::UserService;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub(crate) struct AppContext {
    pub config: OAuthConfig,
    pub token_svc: Arc<TokenService>,
    pub user_svc: Arc<UserService>,
    pub client_svc: Arc<ClientService>,
    pub pat_svc: Arc<PatService>,
}

impl AppContext {
    pub fn build(
        config: OAuthConfig,
        db: DatabaseConnection,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        let factory = SeaOrmDataFactory::new(db);
        Self {
            token_svc: Self::token_service(&config, &factory),
            user_svc: Arc::new(
                UserService::new(factory.user_repository()).with_event_bus(event_bus.clone()),
            ),
            client_svc: Arc::new(
                ClientService::new(factory.client_repository()).with_event_bus(event_bus.clone()),
            ),
            pat_svc: Arc::new(PatService::new(factory.pat_repository()).with_event_bus(event_bus)),
            config,
        }
    }

    pub fn token_service(config: &OAuthConfig, factory: &SeaOrmDataFactory) -> Arc<TokenService> {
        Arc::new(TokenService::new(
            factory.user_repository(),
            factory.token_repository(),
            factory.client_repository(),
            factory.auth_code_repository(),
            factory.pat_repository(),
            config.clone(),
        ))
    }
}
