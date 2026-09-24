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

use axum::Router;
use common::config::services::SsiAuthConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::errors::Outcome;

use crate::data::migrations::get_auth_migrations;
use crate::http::AuthRouter;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

pub struct AuthModule {
    ctx: AppContext,
}

impl AuthModule {
    pub async fn compose(config: &SsiAuthConfig, root: &RootContext) -> Outcome<Self> {
        Ok(Self::new(AppContext::build(config, root).await?))
    }

    pub(crate) fn new(ctx: AppContext) -> Self {
        Self { ctx }
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
