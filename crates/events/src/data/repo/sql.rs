/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::notification;
use crate::data::entities::subscription;

use crate::data::entities::notification::Model;
use crate::data::repo::{
    EditSubscription, EventRepoErrors, EventsRepoFactory, NewNotification, NewSubscription,
    NotificationRepo, SubscriptionRepo,
};
use async_trait::async_trait;
use common::utils::get_urn;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use urn::Urn;

pub struct EventsRepoForSql {
    db_connection: DatabaseConnection,
}

impl EventsRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

impl EventsRepoFactory for EventsRepoForSql {
    fn create_repo(db_connection: DatabaseConnection) -> Self
    where
        Self: Sized,
    {
        Self::new(db_connection)
    }
}

#[async_trait]
impl SubscriptionRepo for EventsRepoForSql {
    async fn get_all_subscriptions(&self) -> Result<Vec<subscription::Model>, EventRepoErrors> {
        let subscriptions = subscription::Entity::find().all(&self.db_connection).await;
        match subscriptions {
            Ok(subscriptions) => Ok(subscriptions),
            Err(e) => Err(EventRepoErrors::ErrorFetchingSubscription(e.to_string())),
        }
    }

    async fn get_subscription_by_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Option<subscription::Model>, EventRepoErrors> {
        let subscription_id = subscription_id.to_string();
        let subscriptions = subscription::Entity::find_by_id(subscription_id)
            .one(&self.db_connection)
            .await;
        match subscriptions {
            Ok(subscriptions) => Ok(subscriptions),
            Err(e) => Err(EventRepoErrors::ErrorFetchingSubscription(e.to_string())),
        }
    }

    async fn get_subscription_by_callback_string(
        &self,
        callback_string: String,
    ) -> Result<Option<subscription::Model>, EventRepoErrors> {
        let subscriptions = subscription::Entity::find()
            .filter(subscription::Column::CallbackAddress.eq(callback_string))
            .one(&self.db_connection)
            .await;
        match subscriptions {
            Ok(subscriptions) => Ok(subscriptions),
            Err(e) => Err(EventRepoErrors::ErrorFetchingSubscription(e.to_string())),
        }
    }

    async fn put_subscription_by_id(
        &self,
        subscription_id: Urn,
        edit_subscription: EditSubscription,
    ) -> Result<subscription::Model, EventRepoErrors> {
        let old_model = self.get_subscription_by_id(subscription_id.clone()).await?;
        let old_model = match old_model {
            Some(old_model) => old_model,
            None => return Err(EventRepoErrors::SubscriptionNotFound),
        };

        let _subscription_id = subscription_id.to_string();
        let mut old_active_model: subscription::ActiveModel = old_model.into();
        if let Some(callback_address) = edit_subscription.callback_address {
            old_active_model.callback_address = ActiveValue::Set(callback_address);
        }
        if let Some(catalog) = edit_subscription.catalog {
            old_active_model.catalog = ActiveValue::Set(catalog);
        }
        if let Some(transfer_process) = edit_subscription.transfer_process {
            old_active_model.transfer_process = ActiveValue::Set(transfer_process);
        }
        if let Some(contract_negotiation_process) = edit_subscription.contract_negotiation_process {
            old_active_model.contract_negotiation_process =
                ActiveValue::Set(contract_negotiation_process);
        }
        if let Some(data_plane) = edit_subscription.data_plane {
            old_active_model.data_plane = ActiveValue::Set(data_plane);
        }
        if let Some(active) = edit_subscription.active {
            old_active_model.active = ActiveValue::Set(active);
        }
        if let Some(expiration_time) = edit_subscription.expiration_time {
            old_active_model.expiration_time = ActiveValue::Set(Option::from(expiration_time));
        }
        old_active_model.updated_at =
            ActiveValue::Set(Option::from(chrono::Utc::now().naive_utc()));
        let model = old_active_model.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(e) => Err(EventRepoErrors::ErrorUpdatingSubscription(e.to_string())),
        }
    }

    async fn create_subscription(
        &self,
        new_subscription: NewSubscription,
    ) -> Result<subscription::Model, EventRepoErrors> {
        let model = subscription::ActiveModel {
            id: ActiveValue::Set(get_urn(None).to_string()),
            callback_address: ActiveValue::Set(new_subscription.callback_address),
            transfer_process: ActiveValue::Set(new_subscription.transfer_process),
            contract_negotiation_process: ActiveValue::Set(
                new_subscription.contract_negotiation_process,
            ),
            catalog: ActiveValue::Set(new_subscription.catalog),
            data_plane: ActiveValue::Set(new_subscription.data_plane),
            active: ActiveValue::Set(new_subscription.active),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            updated_at: ActiveValue::Set(None),
            expiration_time: ActiveValue::Set(new_subscription.expiration_time),
        };
        let subscription = subscription::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match subscription {
            Ok(subscription) => Ok(subscription),
            Err(e) => Err(EventRepoErrors::ErrorCreatingSubscription(e.to_string())),
        }
    }

    async fn delete_subscription_by_id(&self, subscription_id: Urn) -> Result<(), EventRepoErrors> {
        let subscription_id = subscription_id.to_string();
        let subscription = subscription::Entity::delete_by_id(subscription_id)
            .exec(&self.db_connection)
            .await;
        match subscription {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(EventRepoErrors::SubscriptionNotFound),
                _ => Ok(()),
            },
            Err(e) => Err(EventRepoErrors::ErrorDeletingSubscription(e.to_string())),
        }
    }
}

#[async_trait]
impl NotificationRepo for EventsRepoForSql {
    async fn get_all_notifications(&self) -> Result<Vec<notification::Model>, EventRepoErrors> {
        let notifications = notification::Entity::find().all(&self.db_connection).await;
        match notifications {
            Ok(notifications) => Ok(notifications),
            Err(e) => Err(EventRepoErrors::ErrorFetchingNotification(e.to_string())),
        }
    }

    async fn get_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Vec<notification::Model>, EventRepoErrors> {
        let _subscription = match self.get_subscription_by_id(subscription_id.clone()).await? {
            Some(subscription) => subscription,
            None => return Err(EventRepoErrors::SubscriptionNotFound),
        };

        let subscription_id = subscription_id.to_string();
        let notifications = notification::Entity::find()
            .filter(notification::Column::SubscriptionId.eq(subscription_id))
            .all(&self.db_connection)
            .await;
        match notifications {
            Ok(notifications) => Ok(notifications),
            Err(e) => Err(EventRepoErrors::ErrorFetchingNotification(e.to_string())),
        }
    }

    async fn get_pending_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Vec<notification::Model>, EventRepoErrors> {
        let _subscription = match self.get_subscription_by_id(subscription_id.clone()).await? {
            Some(subscription) => subscription,
            None => return Err(EventRepoErrors::SubscriptionNotFound),
        };

        let subscription_id = subscription_id.to_string();
        let notifications = notification::Entity::find()
            .filter(notification::Column::SubscriptionId.eq(subscription_id))
            .filter(notification::Column::Status.eq("Pending"))
            .all(&self.db_connection)
            .await;
        match notifications {
            Ok(notifications) => Ok(notifications),
            Err(e) => Err(EventRepoErrors::ErrorFetchingNotification(e.to_string())),
        }
    }

    async fn ack_pending_notifications_by_subscription_id(
        &self,
        subscription_id: Urn,
    ) -> Result<Vec<Model>, EventRepoErrors> {
        let _subscription = match self.get_subscription_by_id(subscription_id.clone()).await? {
            Some(subscription) => subscription,
            None => return Err(EventRepoErrors::SubscriptionNotFound),
        };
        let subscription_id = subscription_id.to_string();
        let subscription_id = subscription_id.to_string();
        let notifications = notification::Entity::update_many()
            .col_expr(notification::Column::Status, Expr::value("Ok"))
            .filter(notification::Column::SubscriptionId.eq(subscription_id))
            .filter(notification::Column::Status.eq("Pending"))
            .exec_with_returning(&self.db_connection)
            .await;
        match notifications {
            Ok(notifications) => Ok(notifications),
            Err(e) => Err(EventRepoErrors::ErrorFetchingNotification(e.to_string())),
        }
    }

    async fn get_notification_by_id(
        &self,
        subscription_id: Urn,
        notification_id: Urn,
    ) -> Result<Option<notification::Model>, EventRepoErrors> {
        let _subscription = match self.get_subscription_by_id(subscription_id.clone()).await? {
            Some(subscription) => subscription,
            None => return Err(EventRepoErrors::SubscriptionNotFound),
        };
        let notification_id = notification_id.to_string();
        let notifications = notification::Entity::find_by_id(notification_id)
            .one(&self.db_connection)
            .await;
        match notifications {
            Ok(notifications) => Ok(notifications),
            Err(e) => Err(EventRepoErrors::ErrorFetchingNotification(e.to_string())),
        }
    }

    async fn create_notification(
        &self,
        subscription_id: Urn,
        new_notification: NewNotification,
    ) -> Result<notification::Model, EventRepoErrors> {
        let _subscription = match self.get_subscription_by_id(subscription_id.clone()).await? {
            Some(subscription) => subscription,
            None => return Err(EventRepoErrors::SubscriptionNotFound),
        };
        let subscription_id = subscription_id.to_string();
        let model = notification::ActiveModel {
            id: ActiveValue::Set(get_urn(None).to_string()),
            timestamp: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            category: ActiveValue::Set(new_notification.category),
            subcategory: ActiveValue::Set(new_notification.subcategory),
            message_type: ActiveValue::Set(new_notification.message_type),
            message_operation: ActiveValue::Set(new_notification.message_operation),
            message_content: ActiveValue::Set(new_notification.message_content),
            status: ActiveValue::Set(new_notification.status),
            subscription_id: ActiveValue::Set(subscription_id),
        };
        let notification = notification::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match notification {
            Ok(notification) => Ok(notification),
            Err(e) => Err(EventRepoErrors::ErrorCreatingNotification(e.to_string())),
        }
    }
}
