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

use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::subscription::DeadLetterStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dead_letter_queue")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub delivery_id: Option<String>,
    pub event_id: String,
    pub subscription_id: String,
    pub topic: String,
    pub callback_address: String,
    pub payload: serde_json::Value,
    pub error_message: String,
    pub attempts: i32,
    pub status: String,
    pub failed_at: chrono::NaiveDateTime,
    pub replayed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    // Map SeaORM model to pure domain DeadLetterRecord entity.
    pub fn into_domain(self) -> Outcome<DeadLetterRecord> {
        let status = DeadLetterStatus::from_str(&self.status)
            .map_err(|e| Errors::db(format!("invalid dead letter status: {e}"), None))?;

        Ok(DeadLetterRecord {
            id: self.id,
            delivery_id: self.delivery_id,
            event_id: self.event_id,
            subscription_id: self.subscription_id,
            topic: self.topic,
            callback_address: self.callback_address,
            payload: self.payload,
            error_message: self.error_message,
            attempts: self.attempts as u32,
            status,
            failed_at: DateTime::from_naive_utc_and_offset(self.failed_at, Utc),
            replayed_at: self
                .replayed_at
                .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
        })
    }
}

impl ActiveModel {
    // Construct SeaORM ActiveModel from domain DeadLetterRecord entity.
    pub fn from_domain(entity: &DeadLetterRecord) -> Self {
        Self {
            id: ActiveValue::Set(entity.id.clone()),
            delivery_id: ActiveValue::Set(entity.delivery_id.clone()),
            event_id: ActiveValue::Set(entity.event_id.clone()),
            subscription_id: ActiveValue::Set(entity.subscription_id.clone()),
            topic: ActiveValue::Set(entity.topic.clone()),
            callback_address: ActiveValue::Set(entity.callback_address.clone()),
            payload: ActiveValue::Set(entity.payload.clone()),
            error_message: ActiveValue::Set(entity.error_message.clone()),
            attempts: ActiveValue::Set(entity.attempts as i32),
            status: ActiveValue::Set(entity.status.as_str().to_string()),
            failed_at: ActiveValue::Set(entity.failed_at.naive_utc()),
            replayed_at: ActiveValue::Set(entity.replayed_at.map(|t| t.naive_utc())),
        }
    }
}
