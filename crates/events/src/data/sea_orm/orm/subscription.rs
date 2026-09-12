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

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use ymir::errors::{Errors, Outcome};

use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic::TopicPattern;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub callback_address: String,
    pub topic_pattern: Option<String>,
    pub secret: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub retry_limit: Option<i32>,
    pub transfer_process: bool,
    pub contract_negotiation_process: bool,
    pub catalog: bool,
    pub data_plane: bool,
    pub active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::notification::Entity")]
    Notifications,
    #[sea_orm(has_many = "super::delivery::Entity")]
    Deliveries,
}

impl Related<super::notification::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Notifications.def()
    }
}

impl Related<super::delivery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deliveries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    // Map SeaORM model to pure domain SubscriptionRecord entity.
    pub fn into_domain(self) -> Outcome<SubscriptionRecord> {
        let pat_str = self.topic_pattern.unwrap_or_else(|| {
            if self.transfer_process {
                "transfer-agent.**".to_string()
            } else if self.catalog {
                "catalog-agent.**".to_string()
            } else if self.contract_negotiation_process {
                "negotiation-agent.**".to_string()
            } else if self.data_plane {
                "dataplane.**".to_string()
            } else {
                "**".to_string()
            }
        });
        let topic_pattern = TopicPattern::new(&pat_str)
            .map_err(|e| Errors::db(format!("invalid topic pattern: {e}"), None))?;

        let headers = self
            .headers
            .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok());

        Ok(SubscriptionRecord {
            id: self.id,
            callback_address: self.callback_address,
            topic_pattern,
            secret: self.secret,
            headers,
            retry_limit: self.retry_limit.map(|n| n as u32),
            active: self.active,
            created_at: DateTime::from_naive_utc_and_offset(self.created_at, Utc),
            updated_at: self
                .updated_at
                .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
            expiration_time: self
                .expiration_time
                .map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
        })
    }
}

impl ActiveModel {
    // Construct SeaORM ActiveModel from domain SubscriptionRecord entity.
    pub fn from_domain(entity: &SubscriptionRecord) -> Self {
        Self {
            id: ActiveValue::Set(entity.id.clone()),
            callback_address: ActiveValue::Set(entity.callback_address.clone()),
            topic_pattern: ActiveValue::Set(Some(entity.topic_pattern.to_string())),
            secret: ActiveValue::Set(entity.secret.clone()),
            headers: ActiveValue::Set(
                entity
                    .headers
                    .as_ref()
                    .and_then(|h| serde_json::to_value(h).ok()),
            ),
            retry_limit: ActiveValue::Set(entity.retry_limit.map(|n| n as i32)),
            transfer_process: ActiveValue::Set(false),
            contract_negotiation_process: ActiveValue::Set(false),
            catalog: ActiveValue::Set(false),
            data_plane: ActiveValue::Set(false),
            active: ActiveValue::Set(entity.active),
            created_at: ActiveValue::Set(entity.created_at.naive_utc()),
            updated_at: ActiveValue::Set(entity.updated_at.map(|t| t.naive_utc())),
            expiration_time: ActiveValue::Set(entity.expiration_time.map(|t| t.naive_utc())),
        }
    }
}
