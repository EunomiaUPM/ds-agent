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
use common::paginated_spec::{Page, Sort};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryTrait,
};
use ymir::errors::{Errors, Outcome};

use crate::data::repo::EventDeadLetterRepo;
use crate::data::sea_orm::orm::dead_letter;
use crate::data::sea_orm::repos::listing::NaiveKeyset;
use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::dead_letter::DeadLetterStatus;
use crate::entities::queries::DeadLetterFilter;

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

    async fn get_dead_letter(
        &self,
        tenant_id: Option<String>,
        id: &str,
    ) -> Outcome<Option<DeadLetterRecord>> {
        let model = dead_letter::Entity::find()
            .filter(dead_letter::Column::Id.eq(id))
            .apply_if(tenant_id, |q, t| {
                q.filter(dead_letter::Column::TenantId.eq(t))
            })
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
        tenant_id: Option<String>,
        filter: &DeadLetterFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<DeadLetterRecord>, u64)> {
        let query = dead_letter::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(dead_letter::Column::TenantId.eq(t))
            })
            .apply_if(filter.status.clone(), |q, s| {
                q.filter(dead_letter::Column::Status.eq(s))
            });

        let total = query
            .clone()
            .count(&self.db)
            .await
            .map_err(|e| Errors::db("failed to count dead letters", Some(Box::new(e))))?;
        let models = NaiveKeyset::apply(
            query,
            page,
            sort,
            dead_letter::Column::FailedAt,
            dead_letter::Column::Id,
        )
        .all(&self.db)
        .await
        .map_err(|e| Errors::db("failed to list dead letters", Some(Box::new(e))))?;

        let list = models
            .into_iter()
            .map(dead_letter::Model::into_domain)
            .collect::<Outcome<Vec<_>>>()?;
        Ok((list, total))
    }

    async fn mark_replayed(&self, tenant_id: &str, id: &str) -> Outcome<()> {
        if let Some(model) = dead_letter::Entity::find()
            .filter(dead_letter::Column::Id.eq(id))
            .filter(dead_letter::Column::TenantId.eq(tenant_id))
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

    async fn delete_dead_letter(&self, tenant_id: Option<String>, id: &str) -> Outcome<()> {
        dead_letter::Entity::delete_many()
            .filter(dead_letter::Column::Id.eq(id))
            .apply_if(tenant_id, |q, t| {
                q.filter(dead_letter::Column::TenantId.eq(t))
            })
            .exec(&self.db)
            .await
            .map_err(|e| Errors::db("failed to delete dead letter", Some(Box::new(e))))?;
        Ok(())
    }
}
