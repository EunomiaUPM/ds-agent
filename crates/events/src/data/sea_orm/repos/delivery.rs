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
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
};
use ymir::errors::{Errors, Outcome};

use crate::data::repo::EventDeliveryRepo;
use crate::data::sea_orm::orm::delivery;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::subscription::DeliveryStatus;

// SeaORM-backed implementation of EventDeliveryRepo.
#[derive(Clone)]
pub struct SeaOrmDeliveryRepo {
    db: DatabaseConnection,
}

impl SeaOrmDeliveryRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventDeliveryRepo for SeaOrmDeliveryRepo {
    async fn create_delivery(&self, d: &EventDeliveryRecord) -> Outcome<EventDeliveryRecord> {
        let active = delivery::ActiveModel::from_domain(d);
        active
            .insert(&self.db)
            .await
            .map_err(|e| Errors::db("failed to create delivery record", Some(Box::new(e))))?;
        Ok(d.clone())
    }

    async fn get_delivery(&self, id: &str) -> Outcome<Option<EventDeliveryRecord>> {
        let model = delivery::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query delivery record", Some(Box::new(e))))?;

        match model {
            Some(m) => Ok(Some(m.into_domain()?)),
            None => Ok(None),
        }
    }

    async fn get_due_retries(
        &self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Outcome<Vec<EventDeliveryRecord>> {
        let models = delivery::Entity::find()
            .filter(delivery::Column::Status.eq(DeliveryStatus::Failed.as_str()))
            .filter(delivery::Column::NextRetryAt.lte(now.naive_utc()))
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query due retries", Some(Box::new(e))))?;

        let mut list = Vec::with_capacity(models.len());
        for m in models {
            list.push(m.into_domain()?);
        }
        Ok(list)
    }

    async fn mark_delivered(&self, id: &str, attempts: u32, status_code: u16) -> Outcome<()> {
        if let Some(model) = delivery::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to find delivery", Some(Box::new(e))))?
        {
            let mut active: delivery::ActiveModel = model.into();
            active.status = ActiveValue::Set(DeliveryStatus::Delivered.as_str().to_string());
            active.attempts = ActiveValue::Set(attempts as i32);
            active.last_attempt_at = ActiveValue::Set(Some(Utc::now().naive_utc()));
            active.delivered_at = ActiveValue::Set(Some(Utc::now().naive_utc()));
            active.response_status_code = ActiveValue::Set(Some(status_code as i32));
            active.error_message = ActiveValue::Set(None);
            active.next_retry_at = ActiveValue::Set(None);
            active
                .update(&self.db)
                .await
                .map_err(|e| Errors::db("failed to update delivery", Some(Box::new(e))))?;
        }
        Ok(())
    }

    async fn record_failed_attempt(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: Option<DateTime<Utc>>,
        error: &str,
        status_code: Option<u16>,
    ) -> Outcome<()> {
        if let Some(model) = delivery::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to find delivery", Some(Box::new(e))))?
        {
            let mut active: delivery::ActiveModel = model.into();
            active.status = ActiveValue::Set(DeliveryStatus::Failed.as_str().to_string());
            active.attempts = ActiveValue::Set(attempts as i32);
            active.last_attempt_at = ActiveValue::Set(Some(Utc::now().naive_utc()));
            active.next_retry_at = ActiveValue::Set(next_retry_at.map(|t| t.naive_utc()));
            active.error_message = ActiveValue::Set(Some(error.to_string()));
            active.response_status_code = ActiveValue::Set(status_code.map(|s| s as i32));
            active
                .update(&self.db)
                .await
                .map_err(|e| Errors::db("failed to update delivery", Some(Box::new(e))))?;
        }
        Ok(())
    }

    async fn mark_dead_letter(&self, id: &str) -> Outcome<()> {
        if let Some(model) = delivery::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to find delivery", Some(Box::new(e))))?
        {
            let mut active: delivery::ActiveModel = model.into();
            active.status = ActiveValue::Set(DeliveryStatus::DeadLetter.as_str().to_string());
            active.next_retry_at = ActiveValue::Set(None);
            active
                .update(&self.db)
                .await
                .map_err(|e| Errors::db("failed to update delivery", Some(Box::new(e))))?;
        }
        Ok(())
    }

    async fn list_by_event(&self, event_id: &str) -> Outcome<Vec<EventDeliveryRecord>> {
        let models = delivery::Entity::find()
            .filter(delivery::Column::EventId.eq(event_id))
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list deliveries", Some(Box::new(e))))?;

        let mut list = Vec::with_capacity(models.len());
        for m in models {
            list.push(m.into_domain()?);
        }
        Ok(list)
    }
}
