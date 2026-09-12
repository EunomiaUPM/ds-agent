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

pub mod error;

pub mod dispatcher {
    pub use crate::services::event_bus::dispatcher::*;
}
pub mod envelope {
    pub use crate::entities::envelope::*;
    pub use crate::entities::topic::*;
}
pub mod into_event {
    pub use crate::entities::traits::*;
}
pub mod policy {
    pub use crate::services::event_bus::policy::*;
}
pub mod worker {
    pub use crate::services::event_bus::worker::*;
}

pub use crate::entities::envelope::EventEnvelope;
pub use crate::entities::topic::{Topic, TopicPattern};
pub use crate::entities::traits::{Event, IntoEvent};
pub use crate::services::event_bus::{
    DeadLetterView, DeliveryView, EventBus, EventBusTrait, EventDispatcher, EventPublisherTrait,
    EventView, RetryPolicy, RetryWorker, SubscriptionView,
};
pub use error::EventBusError;
