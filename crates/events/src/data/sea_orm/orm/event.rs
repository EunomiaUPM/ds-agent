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
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::entities::envelope::EventEnvelope;
use crate::entities::topic::Topic;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub topic: String,
    pub source_crate: String,
    pub schema_version: i32,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
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
    // Map SeaORM model to pure domain EventEnvelope entity.
    pub fn into_domain(self) -> Outcome<EventEnvelope> {
        let topic = Topic::new(self.topic)
            .map_err(|e| Errors::db(format!("invalid event topic: {e}"), None))?;
        let id = Urn::from_str(&self.id)
            .map_err(|e| Errors::db(format!("invalid event URN: {e}"), None))?;
        let correlation_id = self.correlation_id.and_then(|c| Urn::from_str(&c).ok());

        Ok(EventEnvelope {
            id,
            topic,
            source_crate: self.source_crate,
            schema_version: self.schema_version as u32,
            timestamp: DateTime::from_naive_utc_and_offset(self.timestamp, Utc),
            correlation_id,
            payload: self.payload,
        })
    }
}

impl ActiveModel {
    // Construct SeaORM ActiveModel from domain EventEnvelope entity.
    pub fn from_domain(entity: &EventEnvelope) -> Self {
        Self {
            id: ActiveValue::Set(entity.id.to_string()),
            topic: ActiveValue::Set(entity.topic.to_string()),
            source_crate: ActiveValue::Set(entity.source_crate.clone()),
            schema_version: ActiveValue::Set(entity.schema_version as i32),
            correlation_id: ActiveValue::Set(
                entity.correlation_id.as_ref().map(ToString::to_string),
            ),
            payload: ActiveValue::Set(entity.payload.clone()),
            timestamp: ActiveValue::Set(entity.timestamp.naive_utc()),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
        }
    }
}
