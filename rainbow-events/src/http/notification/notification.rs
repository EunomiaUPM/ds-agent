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

use crate::core::notification::notification_err::NotificationErrors;
use crate::core::notification::RainbowEventsNotificationTrait;
use crate::core::subscription::subscription_types::SubscriptionEntities;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use log::info;
use rainbow_common::utils::get_urn_from_string;
use reqwest::StatusCode;
use std::sync::Arc;

pub struct RainbowEventsNotificationRouter<T> {
    service: Arc<T>,
    entity_type: Option<SubscriptionEntities>,
}
impl<T> RainbowEventsNotificationRouter<T>
where
    T: RainbowEventsNotificationTrait + Send + Sync + 'static,
{
    pub fn new(service: Arc<T>, entity_type: Option<SubscriptionEntities>) -> Self {
        Self { service, entity_type }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/notifications", get(Self::handle_get_all_notifications))
            .route(
                "/subscriptions/:sid/notifications",
                get(Self::handle_get_notifications_by_subscription),
            )
            .route(
                "/subscriptions/:sid/notifications-pending",
                get(Self::handle_get_pending),
            )
            .route(
                "/subscriptions/:sid/ack-notifications-pending",
                post(Self::handle_ack_pending),
            )
            .route(
                "/subscriptions/:sid/notifications/:nid",
                get(Self::handle_get_notification_by_id),
            )
            .with_state((self.service, self.entity_type))
    }
    fn serialize_entity_type(entity: &Option<SubscriptionEntities>) -> String {
        match entity {
            None => "".to_string(),
            Some(entity) => match entity {
                SubscriptionEntities::TransferProcess => "/api/v1/transfers".to_string(),
                SubscriptionEntities::Catalog => "/api/v1/catalog".to_string(),
                SubscriptionEntities::ContractNegotiationProcess => {
                    "/api/v1/contract-negotiations".to_string()
                }
                SubscriptionEntities::DataPlaneProcess => "/api/v1/data-plane".to_string(),
            },
        }
    }
    async fn handle_get_all_notifications(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
    ) -> impl IntoResponse {
        info!("GET {}/notifications", Self::serialize_entity_type(&entity));
        match service.get_all_notifications().await {
            Ok(notifications) => (StatusCode::OK, Json(notifications)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_get_notifications_by_subscription(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path(sid): Path<String>,
    ) -> impl IntoResponse {
        info!(
            "GET {}/subscriptions/{}/notifications",
            Self::serialize_entity_type(&entity),
            sid
        );
        let sid = match get_urn_from_string(&sid) {
            Ok(sid) => sid,
            Err(_) => return NotificationErrors::UrnUuidSchema(sid.to_string()).into_response(),
        };
        match service.get_notifications_by_subscription_id(sid).await {
            Ok(notifications) => (StatusCode::OK, Json(notifications)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_get_pending(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path(sid): Path<String>,
    ) -> impl IntoResponse {
        info!(
            "GET {}/subscriptions/{}/notifications-pending",
            Self::serialize_entity_type(&entity),
            sid
        );
        let sid = match get_urn_from_string(&sid) {
            Ok(sid) => sid,
            Err(_) => return NotificationErrors::UrnUuidSchema(sid.to_string()).into_response(),
        };
        match service.get_pending_notifications_by_subscription_id(sid).await {
            Ok(notifications) => (StatusCode::OK, Json(notifications)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_ack_pending(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path(sid): Path<String>,
    ) -> impl IntoResponse {
        info!(
            "GET {}/subscriptions/{}/ack-notifications-pending",
            Self::serialize_entity_type(&entity),
            sid
        );
        let sid = match get_urn_from_string(&sid) {
            Ok(sid) => sid,
            Err(_) => return NotificationErrors::UrnUuidSchema(sid.to_string()).into_response(),
        };
        match service.ack_pending_notifications_by_subscription_id(sid).await {
            Ok(notifications) => (StatusCode::ACCEPTED, Json(notifications)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_get_notification_by_id(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path((sid, nid)): Path<(String, String)>,
    ) -> impl IntoResponse {
        info!(
            "GET {}/subscriptions/{}/notifications/{}",
            Self::serialize_entity_type(&entity),
            sid,
            nid
        );
        let sid = match get_urn_from_string(&sid) {
            Ok(sid) => sid,
            Err(_) => return NotificationErrors::UrnUuidSchema(sid.to_string()).into_response(),
        };
        let nid = match get_urn_from_string(&nid) {
            Ok(nid) => nid,
            Err(_) => return NotificationErrors::UrnUuidSchema(nid.to_string()).into_response(),
        };
        match service.get_notification_by_id(sid, nid).await {
            Ok(notifications) => (StatusCode::OK, Json(notifications)).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
