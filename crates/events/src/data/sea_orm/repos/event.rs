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
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::data::repo::EventStoreRepo;
use crate::data::sea_orm::orm::event;
use crate::entities::envelope::EventEnvelope;
use crate::entities::topic::TopicPattern;

// SeaORM-backed implementation of EventStoreRepo.
#[derive(Clone)]
pub struct SeaOrmEventRepo {
    db: DatabaseConnection,
}

impl SeaOrmEventRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventStoreRepo for SeaOrmEventRepo {
    async fn insert_event(&self, ev: &EventEnvelope) -> Outcome<()> {
        let active = event::ActiveModel::from_domain(ev);
        active
            .insert(&self.db)
            .await
            .map_err(|e| Errors::db("failed to insert event", Some(Box::new(e))))?;
        Ok(())
    }

    async fn get_event_by_id(&self, id: &Urn) -> Outcome<Option<EventEnvelope>> {
        let model = event::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query event", Some(Box::new(e))))?;

        match model {
            Some(m) => Ok(Some(m.into_domain()?)),
            None => Ok(None),
        }
    }

    async fn list_events(
        &self,
        topic: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<EventEnvelope>> {
        let pattern = topic.and_then(|t| TopicPattern::new(t).ok());
        let mut query = event::Entity::find();

        if let Some(ref pat) = pattern {
            if pat.as_str() == "*" || pat.as_str() == "**" {
                // Match all events.
            } else if pat.has_wildcard() {
                let prefix = pat
                    .as_str()
                    .split('*')
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(['.', ':']);
                if !prefix.is_empty() {
                    query = query.filter(event::Column::Topic.starts_with(prefix));
                }
            } else {
                query = query.filter(event::Column::Topic.eq(pat.as_str()));
            }
        }

        let models = query
            .order_by(event::Column::CreatedAt, Order::Desc)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list events", Some(Box::new(e))))?;

        let mut envelopes = Vec::with_capacity(models.len());
        for m in models {
            let env = m.into_domain()?;
            if let Some(ref pat) = pattern {
                if !pat.matches(&env.topic) {
                    continue;
                }
            }
            envelopes.push(env);
        }
        Ok(envelopes)
    }
}
