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

use auth::data::migrations::get_auth_migrations;
use axum::Router;
use catalog_agent::get_catalog_migrations;
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::service_module::ServiceModuleTrait;
use connector::get_connector_migrations;
use dataplane::get_dataplane_migrations;
use events::data::migrations::get_events_migrations;
use keystore::KeystoreModule;
use negotiation_agent::get_negotiation_agent_migrations;
use oauth::get_oauth_migrations;
use oauth::setup::OAuthModule;
use sea_orm_migration::MigrationTrait;
use std::sync::Arc;
use transfer_agent_ref::setup::TransferAgentModule;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

use crate::setup::context::CoreContext;

/// A thin [`ServiceModuleTrait`] wrapper around one agent's already-built HTTP
/// router, for agents that are still exposed as `create_*_http_router` functions
/// rather than as real modules. It lets those routers sit in the same
/// [`ModuleGroup`] as the migrated ones until each agent grows its own module.
struct ToBeDeprecatedRouterModule {
    name: &'static str,
    prefix: String,
    router: Router,
}

impl ToBeDeprecatedRouterModule {
    fn merged(name: &'static str, router: Router) -> Self {
        Self {
            name,
            prefix: String::new(),
            router,
        }
    }
}

impl ServiceModuleTrait for ToBeDeprecatedRouterModule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((self.prefix.clone(), self.router.clone()))
    }
}

pub struct MonolithModule {
    group: ModuleGroup,
}

impl MonolithModule {
    pub async fn compose(config: &ApplicationConfig, vault: Arc<VaultService>) -> Outcome<Self> {
        let ctx = CoreContext::build(config, vault.clone()).await?;

        let transfer_cfg = config.transfer();
        let transfer = TransferAgentModule::compose(&transfer_cfg, &vault).await?;
        let oauth_db = vault.get_db_connection(transfer_cfg.common()).await?;
        let oauth = OAuthModule::new(transfer_cfg.common().clone().into(), oauth_db);

        let keystore =
            KeystoreModule::build(config.monolith(), Arc::new(config.clone()), vault.clone()).await;

        // Preserve the previous `create_core_router` mount layout: each agent
        // merged at the root, keystore nested under `{api}/keystore`.
        let group = ModuleGroup::new("monolith")
            .register(ToBeDeprecatedRouterModule::merged(
                "catalog-agent",
                ctx.catalog_router,
            ))
            .register(ToBeDeprecatedRouterModule::merged("auth", ctx.auth_router))
            .register(ToBeDeprecatedRouterModule::merged(
                "negotiation-agent",
                ctx.negotiation_router,
            ))
            .register(oauth)
            .register(transfer)
            .register(ToBeDeprecatedRouterModule::merged(
                "gateway",
                ctx.gateway_router,
            ))
            .register(keystore);

        Ok(Self { group })
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            get_catalog_migrations(),
            get_connector_migrations(),
            get_negotiation_agent_migrations(),
            get_events_migrations(),
            get_auth_migrations(),
            get_dataplane_migrations(),
            get_oauth_migrations(),
            TransferAgentModule::migrations(),
            KeystoreModule::migrations(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl ServiceModuleTrait for MonolithModule {
    fn name(&self) -> &'static str {
        "monolith"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        self.group.http()
    }
}
