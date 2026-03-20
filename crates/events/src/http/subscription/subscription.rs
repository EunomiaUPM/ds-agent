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

use crate::core::subscription::subscription_err::SubscriptionErrors;
use crate::core::subscription::subscription_types::{
    EventsSubscriptionCreationRequest, SubscriptionEntities,
};
use crate::core::subscription::EventsSubscriptionTrait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use common::utils::get_urn_from_string;
use reqwest::StatusCode;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

pub struct EventsSubscriptionRouter<T>
where
    T: EventsSubscriptionTrait + Send + Sync,
{
    service: Arc<T>,
    entity_type: Option<SubscriptionEntities>,
}

#[derive(Debug, Deserialize)]
struct SubscriptionQueryParams {
    callback_address: Option<String>,
}

impl<T> EventsSubscriptionRouter<T>
where
    T: EventsSubscriptionTrait + Send + Sync + 'static,
{
    pub fn new(service: Arc<T>, entity_type: Option<SubscriptionEntities>) -> Self {
        Self {
            service,
            entity_type,
        }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/subscriptions", get(Self::handle_get_all_subscriptions))
            .route(
                "/subscriptions/:id",
                get(Self::handle_get_subscription_by_id),
            )
            .route(
                "/subscriptions/:id",
                put(Self::handle_put_subscription_by_id),
            )
            .route("/subscriptions", post(Self::handle_post_subscription_by_id))
            .route(
                "/subscriptions/:id",
                delete(Self::handle_delete_subscription_by_id),
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
    async fn handle_get_all_subscriptions(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        query: Query<SubscriptionQueryParams>,
    ) -> impl IntoResponse {
        let cb = query.callback_address.clone();
        if cb.is_some() {
            info!(
                "GET {}/subscriptions?callback_address={}",
                Self::serialize_entity_type(&entity),
                cb.clone().unwrap()
            );
            match service.get_subscription_by_callback_url(cb.unwrap()).await {
                Ok(subscriptions) => (StatusCode::OK, Json(subscriptions)).into_response(),
                Err(e) => e.into_response(),
            }
        } else {
            info!("GET {}/subscriptions", Self::serialize_entity_type(&entity));
            match service.get_all_subscriptions().await {
                Ok(subscriptions) => (StatusCode::OK, Json(subscriptions)).into_response(),
                Err(e) => e.into_response(),
            }
        }
    }
    async fn handle_get_subscription_by_id(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        info!("mal");
        info!(
            "GET {}/subscriptions/{}",
            Self::serialize_entity_type(&entity),
            id
        );
        let id = match get_urn_from_string(&id) {
            Ok(id) => id,
            Err(_) => return SubscriptionErrors::UrnUuidSchema(id.to_string()).into_response(),
        };
        match service.get_subscription_by_id(id).await {
            Ok(subscriptions) => (StatusCode::OK, Json(subscriptions)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_put_subscription_by_id(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path(id): Path<String>,
        input: Result<Json<EventsSubscriptionCreationRequest>, JsonRejection>,
    ) -> impl IntoResponse {
        info!(
            "PUT {}/subscriptions/{}",
            Self::serialize_entity_type(&entity),
            id
        );
        let id = match get_urn_from_string(&id) {
            Ok(id) => id,
            Err(_) => return SubscriptionErrors::UrnUuidSchema(id.to_string()).into_response(),
        };
        let input = match input {
            Ok(input) => input.0,
            Err(err) => return SubscriptionErrors::JsonRejection(err).into_response(),
        };
        match service.put_subscription_by_id(id, input).await {
            Ok(subscriptions) => (StatusCode::ACCEPTED, Json(subscriptions)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_post_subscription_by_id(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        input: Result<Json<EventsSubscriptionCreationRequest>, JsonRejection>,
    ) -> impl IntoResponse {
        info!(
            "POST {}/subscriptions",
            Self::serialize_entity_type(&entity)
        );
        let input = match input {
            Ok(input) => input.0,
            Err(err) => return SubscriptionErrors::JsonRejection(err).into_response(),
        };
        match service.create_subscription(input, entity.unwrap()).await {
            Ok(subscriptions) => (StatusCode::CREATED, Json(subscriptions)).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_delete_subscription_by_id(
        State((service, entity)): State<(Arc<T>, Option<SubscriptionEntities>)>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        info!(
            "DELETE {}/subscriptions/{}",
            Self::serialize_entity_type(&entity),
            id
        );
        let id = match get_urn_from_string(&id) {
            Ok(id) => id,
            Err(_) => return SubscriptionErrors::UrnUuidSchema(id.to_string()).into_response(),
        };
        match service.delete_subscription_by_id(id).await {
            Ok(subscriptions) => (StatusCode::NO_CONTENT, Json(subscriptions)).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
