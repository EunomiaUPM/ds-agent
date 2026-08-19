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
use axum::Router;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::service_module::ServiceModuleTrait;
use oauth::setup::OAuthModule;
use sea_orm_migration::MigrationTrait;
use std::sync::Arc;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;

use crate::SERVICE_NAME;
use crate::protocols::dsp::setup::DspModule;
use crate::setup::admin_module::TransferAdminModule;
use crate::setup::context::AppContext;

/// The transfer agent as one composable [`ServiceModuleTrait`]: it hosts OAuth,
/// the DSP transfer protocol, and its own transfer-processes / transfer-messages
/// API, and delegates every plane to an internal [`ModuleGroup`]. Because it is
/// itself a module, it drops straight into any [`ServiceComposer`] — standalone
/// in this crate's boot, or alongside other agents in the monolith.
pub struct TransferAgentModule {
    ctx: AppContext,
}

impl TransferAgentModule {
    pub async fn compose(config: &TransferConfig, vault: &VaultService) -> Outcome<Self> {
        Ok(Self {
            ctx: AppContext::build(config, vault).await?,
        })
    }

    fn modules(&self) -> ModuleGroup {
        ModuleGroup::new(SERVICE_NAME)
            .register(OAuthModule::new(
                self.ctx.config.common().clone().into(),
                self.ctx.db.clone(),
            ))
            .register(DspModule::new(Arc::new(self.ctx.clone())))
            .register(TransferAdminModule::new(Arc::new(self.ctx.clone())))
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            oauth::get_oauth_migrations(),
            crate::data::sea_orm::migrations::get_migrations(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl ServiceModuleTrait for TransferAgentModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        self.modules().http()
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        self.modules().grpc(routes);
    }
}
