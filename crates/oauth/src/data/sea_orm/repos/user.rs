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
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::user::{UserRepository, UserRepositoryError};
use crate::data::sea_orm::orm::user as orm;
use crate::entities::query::{Page, Sort, UserFilter};
use crate::entities::role::RbacRole;
use crate::entities::user::User;

pub(crate) struct SeaOrmUserRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn decode_cursor(&self, cursor: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ()> {
        Cursor::decode_timestamp(cursor).map_err(|_| ())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<orm::Entity>,
        filter: &UserFilter,
    ) -> sea_orm::Select<orm::Entity> {
        if let Some(ref tenant_id) = filter.tenant_id {
            q = q.filter(orm::Column::TenantId.eq(tenant_id.as_str()));
        }
        if let Some(role) = filter.role {
            q = q.filter(orm::Column::Role.eq(role.to_string()));
        }
        if let Some(ref email) = filter.email {
            q = q.filter(orm::Column::Email.contains(email.as_str()));
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
impl UserRepository for SeaOrmUserRepository {
    async fn get_all(&self, filter: &UserFilter, page: &Page, sort: &Sort) -> Outcome<Vec<User>> {
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
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn count(&self, filter: &UserFilter) -> Outcome<u64> {
        Self::apply_base_filters(orm::Entity::find(), filter)
            .count(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())
    }

    async fn get_by_tenant_id(&self, tenant_id: &str) -> Outcome<Option<User>> {
        orm::Entity::find_by_id(tenant_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn get_by_email(&self, email: &str) -> Outcome<Option<User>> {
        orm::Entity::find()
            .filter(orm::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn create(&self, user: &User) -> Outcome<User> {
        orm::ActiveModel::from_domain(user)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())
            .and_then(orm::Model::into_domain)
    }

    async fn patch(
        &self,
        tenant_id: &str,
        email: Option<String>,
        role: Option<RbacRole>,
        extra_fields: Option<serde_json::Value>,
    ) -> Outcome<User> {
        let existing = orm::Entity::find_by_id(tenant_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())?
            .ok_or_else(|| UserRepositoryError::NotFound.into_errors())?;

        let mut active: orm::ActiveModel = existing.into();
        if let Some(e) = email {
            active.email = Set(e);
        }
        if let Some(r) = role {
            active.role = Set(r.to_string());
        }
        if let Some(f) = extra_fields {
            active.extra_fields = Set(f);
        }

        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())
            .and_then(orm::Model::into_domain)
    }

    async fn delete(&self, tenant_id: &str) -> Outcome<()> {
        orm::Entity::delete_by_id(tenant_id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| UserRepositoryError::Db(Box::new(e)).into_errors())?;
        Ok(())
    }
}
