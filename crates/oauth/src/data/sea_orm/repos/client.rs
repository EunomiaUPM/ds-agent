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

use std::sync::Arc;

use common::paginated_spec::Cursor;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::client::{ClientRepository, ClientRepositoryError};
use crate::data::sea_orm::orm::client as orm;
use crate::entities::client::Client;
use crate::entities::query::{ClientFilter, Page, Sort};

pub(crate) struct SeaOrmClientRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmClientRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn decode_cursor(&self, cursor: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ()> {
        Cursor::decode_timestamp(cursor).map_err(|_| ())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<orm::Entity>,
        filter: &ClientFilter,
    ) -> sea_orm::Select<orm::Entity> {
        if let Some(role) = filter.role {
            q = q.filter(orm::Column::Role.eq(role.to_string()));
        }
        if let Some(ref search) = filter.search {
            q = q.filter(
                orm::Column::ClientName
                    .contains(search.as_str())
                    .or(orm::Column::ClientId.contains(search.as_str())),
            );
        }
        if let Some(after) = filter.created_after {
            q = q.filter(orm::Column::CreatedAt.gt(after));
        }
        if let Some(before) = filter.created_before {
            q = q.filter(orm::Column::CreatedAt.lt(before));
        }
        q
    }
}

#[async_trait::async_trait]
impl ClientRepository for SeaOrmClientRepository {
    async fn get_all(&self, filter: &ClientFilter, page: &Page, sort: &Sort) -> Outcome<Vec<Client>> {
        let mut q = Self::apply_base_filters(orm::Entity::find(), filter);

        if let Some(ref cursor) = page.cursor {
            if let Ok(cursor_dt) = self.decode_cursor(cursor) {
                q = match sort {
                    Sort::CreatedAtAsc => q.filter(orm::Column::CreatedAt.gt(cursor_dt)),
                    _ => q.filter(orm::Column::CreatedAt.lt(cursor_dt)),
                };
            }
        }

        q = match sort {
            Sort::CreatedAtAsc => q.order_by_asc(orm::Column::CreatedAt),
            _ => q.order_by_desc(orm::Column::CreatedAt),
        };

        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| ClientRepositoryError::Db(Box::new(e)).into_errors())?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn count(&self, filter: &ClientFilter) -> Outcome<u64> {
        Self::apply_base_filters(orm::Entity::find(), filter)
            .count(self.db.as_ref())
            .await
            .map_err(|e| ClientRepositoryError::Db(Box::new(e)).into_errors())
    }

    async fn get_by_client_id(&self, client_id: &str) -> Outcome<Option<Client>> {
        orm::Entity::find_by_id(client_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| ClientRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn create(&self, client: &Client) -> Outcome<Client> {
        orm::ActiveModel::from_domain(client)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| ClientRepositoryError::Db(Box::new(e)).into_errors())
            .and_then(orm::Model::into_domain)
    }

    async fn delete(&self, client_id: &str) -> Outcome<()> {
        orm::Entity::delete_by_id(client_id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| ClientRepositoryError::Db(Box::new(e)).into_errors())?;
        Ok(())
    }
}
