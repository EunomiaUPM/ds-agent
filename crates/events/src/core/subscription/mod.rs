/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::core::subscription::subscription_types::{
    EventsSubscriptionCreationRequest, EventsSubscriptionCreationResponse, SubscriptionEntities,
};
use async_trait::async_trait;
use urn::Urn;
use ymir::errors::Outcome;

pub mod subscription;
pub mod subscription_err;
pub mod subscription_types;

#[mockall::automock]
#[async_trait]
pub trait EventsSubscriptionTrait: Send + Sync {
    async fn get_all_subscriptions(&self) -> Outcome<Vec<EventsSubscriptionCreationResponse>>;
    async fn get_subscription_by_id(
        &self,
        subscription_id: Urn,
    ) -> Outcome<EventsSubscriptionCreationResponse>;
    async fn get_subscription_by_callback_url(
        &self,
        callback_url: String,
    ) -> Outcome<EventsSubscriptionCreationResponse>;
    async fn put_subscription_by_id(
        &self,
        subscription_id: Urn,
        input: EventsSubscriptionCreationRequest,
    ) -> Outcome<EventsSubscriptionCreationResponse>;
    async fn create_subscription(
        &self,
        input: EventsSubscriptionCreationRequest,
        subscription_type: SubscriptionEntities,
    ) -> Outcome<EventsSubscriptionCreationResponse>;
    async fn delete_subscription_by_id(&self, subscription_id: Urn) -> Outcome<()>;
}
