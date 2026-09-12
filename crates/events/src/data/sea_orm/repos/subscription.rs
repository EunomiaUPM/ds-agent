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

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repo::EventSubscriptionRepo;
use crate::data::sea_orm::orm::subscription;
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic::{Topic, TopicPattern};

// SeaORM-backed implementation of EventSubscriptionRepo.
#[derive(Clone)]
pub struct SeaOrmSubscriptionRepo {
    db: DatabaseConnection,
}

impl SeaOrmSubscriptionRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventSubscriptionRepo for SeaOrmSubscriptionRepo {
    async fn create_subscription(&self, dto: CreateSubscriptionDto) -> Outcome<SubscriptionRecord> {
        let id = format!("urn:uuid:{}", Uuid::new_v4());
        let pattern = TopicPattern::new(&dto.topic_pattern)
            .map_err(|e| Errors::format(BadFormat::Received, e, None))?;

        let headers_val = dto
            .headers
            .as_ref()
            .map(|h| serde_json::to_value(h).unwrap_or_default());

        let active = subscription::ActiveModel {
            id: ActiveValue::Set(id.clone()),
            callback_address: ActiveValue::Set(dto.callback_address.clone()),
            topic_pattern: ActiveValue::Set(Some(dto.topic_pattern)),
            secret: ActiveValue::Set(dto.secret.clone()),
            headers: ActiveValue::Set(headers_val),
            retry_limit: ActiveValue::Set(dto.retry_limit.map(|r| r as i32)),
            transfer_process: ActiveValue::Set(false),
            contract_negotiation_process: ActiveValue::Set(false),
            catalog: ActiveValue::Set(false),
            data_plane: ActiveValue::Set(false),
            active: ActiveValue::Set(true),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
            updated_at: ActiveValue::Set(None),
            expiration_time: ActiveValue::Set(dto.expiration_time.map(|e| e.naive_utc())),
        };

        active
            .insert(&self.db)
            .await
            .map_err(|e| Errors::db("failed to create subscription", Some(Box::new(e))))?;

        Ok(SubscriptionRecord {
            id,
            callback_address: dto.callback_address,
            topic_pattern: pattern,
            secret: dto.secret,
            headers: dto.headers,
            retry_limit: dto.retry_limit,
            active: true,
            created_at: Utc::now(),
            updated_at: None,
            expiration_time: dto.expiration_time,
        })
    }

    async fn get_subscription(&self, id: &str) -> Outcome<Option<SubscriptionRecord>> {
        let model = subscription::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query subscription", Some(Box::new(e))))?;

        match model {
            Some(m) => Ok(Some(m.into_domain()?)),
            None => Ok(None),
        }
    }

    async fn list_subscriptions(&self) -> Outcome<Vec<SubscriptionRecord>> {
        let models = subscription::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list subscriptions", Some(Box::new(e))))?;

        let mut results = Vec::with_capacity(models.len());
        for m in models {
            results.push(m.into_domain()?);
        }
        Ok(results)
    }

    async fn update_subscription(
        &self,
        id: &str,
        dto: UpdateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord> {
        let model = subscription::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to find subscription", Some(Box::new(e))))?
            .ok_or_else(|| Errors::missing_resource(id, "subscription not found", None))?;

        let mut active: subscription::ActiveModel = model.into();
        if let Some(addr) = dto.callback_address {
            active.callback_address = ActiveValue::Set(addr);
        }
        if let Some(pat) = dto.topic_pattern {
            active.topic_pattern = ActiveValue::Set(Some(pat));
        }
        if dto.secret.is_some() {
            active.secret = ActiveValue::Set(dto.secret);
        }
        if let Some(headers) = dto.headers {
            active.headers =
                ActiveValue::Set(Some(serde_json::to_value(headers).unwrap_or_default()));
        }
        if dto.retry_limit.is_some() {
            active.retry_limit = ActiveValue::Set(dto.retry_limit.map(|r| r as i32));
        }
        if let Some(act) = dto.active {
            active.active = ActiveValue::Set(act);
        }
        if dto.expiration_time.is_some() {
            active.expiration_time = ActiveValue::Set(dto.expiration_time.map(|e| e.naive_utc()));
        }
        active.updated_at = ActiveValue::Set(Some(Utc::now().naive_utc()));

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| Errors::db("failed to update subscription", Some(Box::new(e))))?;

        updated.into_domain()
    }

    async fn delete_subscription(&self, id: &str) -> Outcome<()> {
        subscription::Entity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| Errors::db("failed to delete subscription", Some(Box::new(e))))?;
        Ok(())
    }

    async fn get_matching_subscriptions(&self, topic: &Topic) -> Outcome<Vec<SubscriptionRecord>> {
        let all = self.list_subscriptions().await?;
        Ok(all.into_iter().filter(|s| s.matches(topic)).collect())
    }
}
