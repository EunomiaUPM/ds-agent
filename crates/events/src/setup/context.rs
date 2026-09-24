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

use common::module_loader::root_context::RootContext;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::SeaOrmDataFactory;
use crate::services::event_bus::policy::RetryPolicy;
use crate::services::event_bus::worker::RetryWorker;
use crate::services::event_bus::EventBus;

/// The event bus and the retry worker that redelivers what it failed to dispatch.
#[derive(Clone)]
pub(crate) struct AppContext {
    pub event_bus: Arc<EventBus>,
    pub retry_worker: RetryWorker,
}

impl AppContext {
    pub fn build(root: &RootContext, policy: Option<RetryPolicy>) -> Self {
        let policy = policy.unwrap_or_default();
        let factory = SeaOrmDataFactory::new(root.db.clone());

        let event_repo = factory.event_repository();
        let subscription_repo = factory.subscription_repository();
        let delivery_repo = factory.delivery_repository();
        let dlq_repo = factory.dlq_repository();

        let event_bus = Arc::new(EventBus::new(
            event_repo.clone(),
            subscription_repo.clone(),
            delivery_repo.clone(),
            dlq_repo.clone(),
            policy.clone(),
            1024,
        ));

        let retry_worker = RetryWorker::new(
            event_repo.clone(),
            subscription_repo.clone(),
            delivery_repo.clone(),
            dlq_repo.clone(),
            event_bus.dispatcher(),
            policy,
        );

        Self {
            event_bus,
            retry_worker,
        }
    }
}
