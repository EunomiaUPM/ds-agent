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

//! `subscriptions` table.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use common::oauth::{Owner, RolePath, Visibility};
use common::secret::Secret;
use sea_orm::ActiveValue;
use ymir::errors::{Errors, Outcome};

use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic_pattern::TopicPattern;

/// `subscriptions` row.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub callback_address: String,
    pub topic_pattern: String,
    pub secret: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub retry_limit: Option<i32>,
    pub active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub expiration_time: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::delivery::Entity")]
    Deliveries,
}

impl Related<super::delivery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deliveries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_domain(self) -> Outcome<SubscriptionRecord> {
        let topic_pattern = TopicPattern::new(&self.topic_pattern)
            .map_err(|e| Errors::db(format!("invalid topic pattern: {e}"), None))?;

        let headers = self
            .headers
            .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok());

        Ok(SubscriptionRecord {
            id: self.id,
            owner: Owner::new(self.user_id, self.user_role, self.visibility),
            callback_address: self.callback_address,
            topic_pattern,
            secret: self.secret.map(Secret::new),
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
    pub fn from_domain(entity: &SubscriptionRecord) -> Self {
        Self {
            id: ActiveValue::Set(entity.id.clone()),
            user_id: ActiveValue::Set(entity.owner.user_id.clone()),
            user_role: ActiveValue::Set(entity.owner.role.clone()),
            visibility: ActiveValue::Set(entity.owner.visibility.clone()),
            callback_address: ActiveValue::Set(entity.callback_address.clone()),
            topic_pattern: ActiveValue::Set(entity.topic_pattern.to_string()),
            secret: ActiveValue::Set(entity.secret.clone().map(Secret::into_exposed)),
            headers: ActiveValue::Set(
                entity
                    .headers
                    .as_ref()
                    .and_then(|h| serde_json::to_value(h).ok()),
            ),
            retry_limit: ActiveValue::Set(entity.retry_limit.map(|n| n as i32)),
            active: ActiveValue::Set(entity.active),
            created_at: ActiveValue::Set(entity.created_at.naive_utc()),
            updated_at: ActiveValue::Set(entity.updated_at.map(|t| t.naive_utc())),
            expiration_time: ActiveValue::Set(entity.expiration_time.map(|t| t.naive_utc())),
        }
    }
}
