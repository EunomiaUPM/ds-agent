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
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, Order,
    QueryFilter, QueryOrder, QuerySelect,
};
use ymir::errors::{Errors, Outcome};

use crate::data::repo::EventDeadLetterRepo;
use crate::data::sea_orm::orm::dead_letter;
use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::subscription::DeadLetterStatus;

// SeaORM-backed implementation of EventDeadLetterRepo.
#[derive(Clone)]
pub struct SeaOrmDeadLetterRepo {
    db: DatabaseConnection,
}

impl SeaOrmDeadLetterRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventDeadLetterRepo for SeaOrmDeadLetterRepo {
    async fn create_dead_letter(&self, record: &DeadLetterRecord) -> Outcome<DeadLetterRecord> {
        let active = dead_letter::ActiveModel::from_domain(record);
        active
            .insert(&self.db)
            .await
            .map_err(|e| Errors::db("failed to create dead letter", Some(Box::new(e))))?;
        Ok(record.clone())
    }

    async fn get_dead_letter(&self, id: &str) -> Outcome<Option<DeadLetterRecord>> {
        let model = dead_letter::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query dead letter", Some(Box::new(e))))?;

        match model {
            Some(m) => Ok(Some(m.into_domain()?)),
            None => Ok(None),
        }
    }

    async fn list_dead_letters(
        &self,
        status: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<DeadLetterRecord>> {
        let mut query = dead_letter::Entity::find();
        if let Some(s) = status {
            query = query.filter(dead_letter::Column::Status.eq(s));
        }

        let models = query
            .order_by(dead_letter::Column::FailedAt, Order::Desc)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list dead letters", Some(Box::new(e))))?;

        let mut list = Vec::with_capacity(models.len());
        for m in models {
            list.push(m.into_domain()?);
        }
        Ok(list)
    }

    async fn mark_replayed(&self, id: &str) -> Outcome<()> {
        if let Some(model) = dead_letter::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to find dead letter", Some(Box::new(e))))?
        {
            let mut active: dead_letter::ActiveModel = model.into();
            active.status = ActiveValue::Set(DeadLetterStatus::Replayed.as_str().to_string());
            active.replayed_at = ActiveValue::Set(Some(Utc::now().naive_utc()));
            active
                .update(&self.db)
                .await
                .map_err(|e| Errors::db("failed to update dead letter", Some(Box::new(e))))?;
        }
        Ok(())
    }

    async fn delete_dead_letter(&self, id: &str) -> Outcome<()> {
        dead_letter::Entity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| Errors::db("failed to delete dead letter", Some(Box::new(e))))?;
        Ok(())
    }
}
