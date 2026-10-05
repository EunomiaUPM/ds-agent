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

//! The event bus.

pub mod dispatcher;
pub mod policy;
pub mod service;
pub mod views;
pub mod worker;

use async_trait::async_trait;
use common::oauth::Owner;
use tokio::sync::broadcast;

pub use dispatcher::EventDispatcher;
pub use policy::RetryPolicy;
pub use service::EventBus;
pub use views::{DeadLetterView, DeliveryView, EventView, SubscriptionView};
pub use worker::RetryWorker;

use crate::entities::envelope::EventEnvelope;
use crate::entities::event::Event;
use ymir::errors::Outcome;

/// Publishing envelopes and listening to them in-process.
#[async_trait]
pub trait EventBusTrait: Send + Sync + 'static {
    /// Stores the envelope, broadcasts it and starts a delivery per matching subscription.
    async fn publish(&self, envelope: EventEnvelope) -> Outcome<EventEnvelope>;
    /// New receiver of every envelope published from now on.
    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope>;
}

/// Publishing typed events and raw payloads.
#[async_trait]
pub trait EventPublisherTrait: Send + Sync {
    /// Wraps the event in an envelope and publishes it.
    async fn publish_event<E: Event>(&self, event: E) -> Outcome<EventEnvelope>;

    /// Publishes `payload` under `topic` about a record of `owner`.
    async fn emit_payload(
        &self,
        owner: &Owner,
        topic: &str,
        source: &str,
        payload: &serde_json::Value,
    ) -> Outcome<EventEnvelope>;
}
