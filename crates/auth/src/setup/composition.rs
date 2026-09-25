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

//! SSI auth agent as a composable module: wallet, GNAP gatekeeper, verifier and issuer.

use std::sync::Arc;

use axum::Router;
use common::config::services::SsiAuthConfig;
use common::config::types::traits::CommonConfigTrait;
use common::facades::AuthPorts;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::errors::Outcome;

use crate::data::migrations::get_auth_migrations;
use crate::facades::{MatesLocalFacade, SSIAuthLocalFacade};
use crate::http::AuthRouter;
use crate::setup::context::AppContext;
use crate::setup::seeders::SelfParticipantOnboarder;
use crate::SERVICE_NAME;

pub struct AuthModule {
    ctx: AppContext,
    /// Tenant of the service client, whose token the local facades impersonate.
    service_tenant: String,
}

impl AuthModule {
    pub async fn compose(config: &SsiAuthConfig, root: &RootContext) -> Outcome<Self> {
        Ok(Self::new(
            AppContext::build(config, root).await?,
            config.common().admin_seed.tenant_id.clone(),
        ))
    }

    pub(crate) fn new(ctx: AppContext, service_tenant: String) -> Self {
        Self {
            ctx,
            service_tenant,
        }
    }

    /// Onboards this agent's own wallet as a participant; registered by the monolith only.
    pub fn self_participant_onboarder(&self) -> SelfParticipantOnboarder {
        SelfParticipantOnboarder::new(self.ctx.core.clone(), self.service_tenant.clone())
    }

    /// Auth ports served in-process, for every agent sharing this process.
    pub fn local_ports(&self) -> AuthPorts {
        AuthPorts {
            mates: Arc::new(MatesLocalFacade::new(self.ctx.core.clone())),
            ssi_auth: Arc::new(SSIAuthLocalFacade::new(
                self.ctx.core.clone(),
                self.service_tenant.clone(),
            )),
        }
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        get_auth_migrations()
    }
}

impl ServiceModuleTrait for AuthModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let router =
            AuthRouter::new(self.ctx.core.clone(), self.ctx.oauth_validator.clone()).router();
        Some((String::new(), router))
    }
}
