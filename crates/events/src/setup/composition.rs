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
use common::auth::OauthTokenValidator;
use common::boot::workers::BackgroundWorker;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;

use crate::http::EventsHttpRouter;
use crate::services::event_bus::EventBus;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

/// Events service module integrating migrations and HTTP routes into the modular host.
pub struct EventsModule {
    ctx: Arc<AppContext>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl EventsModule {
    pub fn compose(root: &RootContext) -> Self {
        Self {
            ctx: Arc::new(AppContext::build(root, None)),
            validator: root.validator.clone(),
        }
    }

    /// Bus every other module publishes to; hand it out before registering them.
    pub fn event_bus(&self) -> EventBus {
        (*self.ctx.event_bus).clone()
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
        let router = EventsHttpRouter::build(self.ctx.event_bus.clone(), self.validator.clone());
        Some((format!("/api/v1/{SERVICE_NAME}"), router))
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        vec![Box::new(self.ctx.retry_worker.clone())]
    }
}
