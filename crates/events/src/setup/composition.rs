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
use sea_orm_migration::MigrationTrait;

use crate::http::EventsHttpRouter;
use crate::setup::context::AppContext;

pub const SERVICE_NAME: &str = "events";

/// Events service module integrating migrations and HTTP routes into the modular host.
pub struct EventsModule {
    ctx: Arc<AppContext>,
    validator: Option<Arc<dyn common::auth::OauthTokenValidator>>,
}

impl EventsModule {
    /// Construct a new EventsModule with the given application context.
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            ctx,
            validator: None,
        }
    }

    /// Set an OAuth token validator to enforce AuthHttpMiddleware on HTTP routes.
    pub fn with_token_validator(
        mut self,
        validator: Arc<dyn common::auth::OauthTokenValidator>,
    ) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Return all SeaORM database migrations for the events bus and legacy tables.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::data::migrations::get_events_migrations()
    }
}

impl ServiceModuleTrait for EventsModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let router = match &self.validator {
            Some(validator) => EventsHttpRouter::build_with_validator(
                self.ctx.event_bus.clone(),
                validator.clone(),
            ),
            None => EventsHttpRouter::build(self.ctx.event_bus.clone()),
        };
        Some((format!("/api/v1/{SERVICE_NAME}"), router))
    }
}
