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

#![allow(clippy::result_large_err, clippy::module_inception)]

pub mod data;
pub mod entities;
pub mod http;
pub mod services;
pub mod setup;

pub mod bus;
pub(crate) mod errors;

pub use entities::envelope::EventEnvelope;
pub use entities::topic::{Topic, TopicPattern};
pub use entities::traits::{Event, IntoEvent};
pub use errors::EventBusError;
pub use services::event_bus::{
    DeadLetterView, DeliveryView, EventBus, EventBusTrait, EventDispatcher, EventPublisherTrait,
    EventView, RetryPolicy, RetryWorker, SubscriptionView,
};
