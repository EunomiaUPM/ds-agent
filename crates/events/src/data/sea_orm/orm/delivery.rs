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

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use ymir::errors::{Errors, Outcome};

use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::subscription::DeliveryStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_deliveries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub event_id: String,
    pub subscription_id: String,
    pub status: String,
    pub attempts: i32,
    pub last_attempt_at: Option<chrono::NaiveDateTime>,
    pub next_retry_at: Option<chrono::NaiveDateTime>,
    pub error_message: Option<String>,
    pub response_status_code: Option<i32>,
    pub delivered_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::event::Entity",
        from = "Column::EventId",
        to = "super::event::Column::Id"
    )]
    Event,
    #[sea_orm(
        belongs_to = "super::subscription::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscription::Column::Id"
    )]
    Subscription,
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl Related<super::subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    // Map SeaORM model to pure domain EventDeliveryRecord entity.
    pub fn into_domain(self) -> Outcome<EventDeliveryRecord> {
        let status = DeliveryStatus::from_str(&self.status)
            .map_err(|e| Errors::db(format!("invalid delivery status: {e}"), None))?;

        Ok(EventDeliveryRecord {
            id: self.id,
            event_id: self.event_id,
            subscription_id: self.subscription_id,
            status,
            attempts: self.attempts as u32,
            last_attempt_at: self
                .last_attempt_at
                .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
            next_retry_at: self
                .next_retry_at
                .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
            error_message: self.error_message,
            response_status_code: self.response_status_code.map(|c| c as u16),
            delivered_at: self
                .delivered_at
                .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
            created_at: DateTime::from_naive_utc_and_offset(self.created_at, Utc),
        })
    }
}

impl ActiveModel {
    // Construct SeaORM ActiveModel from domain EventDeliveryRecord entity.
    pub fn from_domain(entity: &EventDeliveryRecord) -> Self {
        Self {
            id: ActiveValue::Set(entity.id.clone()),
            event_id: ActiveValue::Set(entity.event_id.clone()),
            subscription_id: ActiveValue::Set(entity.subscription_id.clone()),
            status: ActiveValue::Set(entity.status.as_str().to_string()),
            attempts: ActiveValue::Set(entity.attempts as i32),
            last_attempt_at: ActiveValue::Set(entity.last_attempt_at.map(|t| t.naive_utc())),
            next_retry_at: ActiveValue::Set(entity.next_retry_at.map(|t| t.naive_utc())),
            error_message: ActiveValue::Set(entity.error_message.clone()),
            response_status_code: ActiveValue::Set(entity.response_status_code.map(|c| c as i32)),
            delivered_at: ActiveValue::Set(entity.delivered_at.map(|t| t.naive_utc())),
            created_at: ActiveValue::Set(entity.created_at.naive_utc()),
        }
    }
}
