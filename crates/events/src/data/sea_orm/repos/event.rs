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
use common::paginated_spec::{Page, Sort};
use sea_orm::sea_query::{BinOper, Expr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryTrait,
};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::data::repo::EventStoreRepo;
use crate::data::sea_orm::orm::event;
use crate::data::sea_orm::repos::listing::NaiveKeyset;
use crate::entities::envelope::EventEnvelope;
use crate::entities::queries::EventFilter;

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

    async fn get_event_by_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<EventEnvelope>> {
        let model = event::Entity::find()
            .filter(event::Column::Id.eq(id.to_string()))
            .apply_if(tenant_id, |q, t| q.filter(event::Column::TenantId.eq(t)))
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
        tenant_id: Option<String>,
        filter: &EventFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<EventEnvelope>, u64)> {
        let mut query = event::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(event::Column::TenantId.eq(t)));
        if let Some(pattern) = filter.topic_pattern()? {
            query = query.filter(
                Expr::col((event::Entity, event::Column::Topic))
                    .binary(BinOper::Custom("~"), pattern.to_sql_regex()),
            );
        }

        let total = query
            .clone()
            .count(&self.db)
            .await
            .map_err(|e| Errors::db("failed to count events", Some(Box::new(e))))?;
        let models = NaiveKeyset::apply(
            query,
            page,
            sort,
            event::Column::Timestamp,
            event::Column::Id,
        )
        .all(&self.db)
        .await
        .map_err(|e| Errors::db("failed to list events", Some(Box::new(e))))?;

        let events = models
            .into_iter()
            .map(event::Model::into_domain)
            .collect::<Outcome<Vec<_>>>()?;
        Ok((events, total))
    }
}
