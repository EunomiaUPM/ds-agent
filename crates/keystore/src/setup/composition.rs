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

//! Keystore as a composable module: parameters, secrets and the config passthrough.

use std::sync::Arc;

use crate::http::KeystoreRouter;
use crate::services::parameters::ParameterStore;
use crate::services::secrets::SecretStore;
use crate::setup::context::AppContext;
use axum::Router;
use common::config::ApplicationConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::ApiConfigTrait;

pub struct KeystoreModule {
    prefix: String,
    ctx: AppContext,
}

impl KeystoreModule {
    pub fn compose(
        config: &ApplicationConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        Self {
            prefix: format!("{}/keystore", config.common().get_api_version()),
            ctx: AppContext::build(config, root, event_bus),
        }
    }

    /// Parameter and secret stores alone, for crates that read the keystore in-process.
    pub fn build_stores(
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> (
        Arc<dyn ParameterStore<serde_json::Value>>,
        Arc<dyn SecretStore>,
    ) {
        AppContext::build_stores(root, event_bus)
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::get_keystore_migrations()
    }
}

impl ServiceModuleTrait for KeystoreModule {
    fn name(&self) -> &'static str {
        "keystore"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let ctx = &self.ctx;
        let router = KeystoreRouter::new(
            ctx.parameter_svc.clone(),
            ctx.secret_svc.clone(),
            ctx.config_svc.clone(),
            ctx.oauth_validator.clone(),
        )
        .router();
        Some((self.prefix.clone(), router))
    }
}
