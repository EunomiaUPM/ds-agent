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

use crate::data::entities::{notification, subscription};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use thiserror::Error;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

pub mod sql;

pub trait EventsRepoFactory: SubscriptionRepo + NotificationRepo + Send + Sync + 'static {
    fn create_repo(db_connection: DatabaseConnection) -> Self
    where
        Self: Sized;
}

pub struct NewSubscription {
    pub callback_address: String,
    pub transfer_process: bool,
    pub contract_negotiation_process: bool,
    pub catalog: bool,
    pub data_plane: bool,
    pub active: bool,
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

pub struct EditSubscription {
    pub callback_address: Option<String>,
    pub transfer_process: Option<bool>,
    pub contract_negotiation_process: Option<bool>,
    pub catalog: Option<bool>,
    pub data_plane: Option<bool>,
    pub active: Option<bool>,
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

impl Default for EditSubscription {
    fn default() -> Self {
        Self {
            callback_address: None,
            transfer_process: None,
            contract_negotiation_process: None,
            catalog: None,
            data_plane: None,
            active: None,
            expiration_time: None,
        }
    }
}

#[async_trait]
pub trait SubscriptionRepo {
    async fn get_all_subscriptions(&self) -> Result<Vec<subscription::Model>, EventRepoErrors>;
    async fn get_subscription_by_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Option<subscription::Model>, EventRepoErrors>;
    async fn get_subscription_by_callback_string(
        &self,
        callback_string: String,
    ) -> Result<Option<subscription::Model>, EventRepoErrors>;
    async fn put_subscription_by_id(
        &self,
        subscription_id: Urn,
        edit_subscription: EditSubscription,
    ) -> Result<subscription::Model, EventRepoErrors>;
    async fn create_subscription(
        &self,
        new_subscription: NewSubscription,
    ) -> Result<subscription::Model, EventRepoErrors>;
    async fn delete_subscription_by_id(&self, subscription_id: Urn) -> Result<(), EventRepoErrors>;
}

pub struct NewNotification {
    pub category: String,
    pub subcategory: String,
    pub message_type: String,
    pub message_operation: String,
    pub message_content: serde_json::Value,
    pub status: String,
}

#[async_trait]
pub trait NotificationRepo {
    async fn get_all_notifications(&self) -> Result<Vec<notification::Model>, EventRepoErrors>;
    async fn get_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Vec<notification::Model>, EventRepoErrors>;
    async fn get_pending_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Vec<notification::Model>, EventRepoErrors>;
    async fn ack_pending_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Vec<notification::Model>, EventRepoErrors>;

    async fn get_notification_by_id(
        &self,
        subscription_id: Urn,
        notification_id: Urn,
    ) -> Result<Option<notification::Model>, EventRepoErrors>;
    async fn create_notification(
        &self,
        subscription_id: Urn,
        new_notification: NewNotification,
    ) -> Result<notification::Model, EventRepoErrors>;
}

#[derive(Debug, Error)]
pub enum EventRepoErrors {
    #[error("Subscription not found")]
    SubscriptionNotFound,
    #[error("Notification not found")]
    NotificationNotFound,

    #[error("Error fetching subscription. {0}")]
    ErrorFetchingSubscription(String),
    #[error("Error creating subscription. {0}")]
    ErrorCreatingSubscription(String),
    #[error("Error deleting subscription. {0}")]
    ErrorDeletingSubscription(String),
    #[error("Error updating subscription. {0}")]
    ErrorUpdatingSubscription(String),

    #[error("Error fetching notification. {0}")]
    ErrorFetchingNotification(String),
    #[error("Error creating notification. {0}")]
    ErrorCreatingNotification(String),
    #[error("Error deleting notification. {0}")]
    ErrorDeletingNotification(String),
    #[error("Error updating notification. {0}")]
    ErrorUpdatingNotification(String),
}

impl RepoIntoErrors for EventRepoErrors {}
