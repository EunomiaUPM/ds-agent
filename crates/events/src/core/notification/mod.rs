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

use crate::core::notification::notification_types::{
    EventsNotificationBroadcastRequest, EventsNotificationCreationRequest,
    EventsNotificationResponse,
};
use async_trait::async_trait;
use urn::Urn;
use ymir::errors::Outcome;

pub mod notification;
pub mod notification_err;
pub mod notification_types;

#[mockall::automock]
#[async_trait]
pub trait EventsNotificationTrait: Send + Sync {
    async fn get_all_notifications(&self) -> Outcome<Vec<EventsNotificationResponse>>;
    async fn get_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Outcome<Vec<EventsNotificationResponse>>;
    async fn get_pending_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Outcome<Vec<EventsNotificationResponse>>;

    async fn ack_pending_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Outcome<Vec<EventsNotificationResponse>>;

    async fn get_notification_by_id(
        &self,
        subscription_id: Urn,
        notification_id: Urn,
    ) -> Outcome<EventsNotificationResponse>;
    async fn create_notification(
        &self,
        subscription_id: Urn,
        input: EventsNotificationCreationRequest,
    ) -> Outcome<EventsNotificationResponse>;

    async fn broadcast_notification(
        &self,
        input: EventsNotificationBroadcastRequest,
    ) -> Outcome<()>;
}
