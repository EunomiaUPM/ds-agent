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

use sea_orm::QueryTrait;
use std::sync::Arc;

use chrono::Utc;
use common::paginated_spec::Cursor;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::pat::{PatRepository, PatRepositoryError};
use crate::data::sea_orm::orm::pat as orm;
use crate::entities::pat::PersonalAccessToken;
use crate::entities::query::{Page, PatFilter, Sort};

pub(crate) struct SeaOrmPatRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmPatRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn decode_cursor(&self, cursor: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ()> {
        Cursor::decode_timestamp(cursor).map_err(|_| ())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<orm::Entity>,
        filter: &PatFilter,
    ) -> sea_orm::Select<orm::Entity> {
        if let Some(ref tenant_id) = filter.tenant_id {
            q = q.filter(orm::Column::TenantId.eq(tenant_id.as_str()));
        }
        if let Some(ref status) = filter.status {
            if status == "active" {
                q = q.filter(orm::Column::Revoked.eq(false));
                q = q.filter(
                    orm::Column::ExpiresAt
                        .is_null()
                        .or(orm::Column::ExpiresAt.gt(chrono::Utc::now())),
                );
            } else if status == "revoked" {
                q = q.filter(orm::Column::Revoked.eq(true));
            }
        }
        if let Some(role) = filter.role {
            q = q.filter(orm::Column::Role.eq(role.to_string()));
        }
        if let Some(ref search) = filter.search {
            q = q.filter(
                orm::Column::Name
                    .contains(search.as_str())
                    .or(orm::Column::TokenPrefix.contains(search.as_str())),
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
impl PatRepository for SeaOrmPatRepository {
    async fn get_all(
        &self,
        filter: &PatFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<PersonalAccessToken>> {
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
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn count(&self, filter: &PatFilter) -> Outcome<u64> {
        Self::apply_base_filters(orm::Entity::find(), filter)
            .count(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())
    }
    async fn create(&self, pat: &PersonalAccessToken) -> Outcome<PersonalAccessToken> {
        orm::ActiveModel::from_domain(pat)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())
            .and_then(orm::Model::into_domain)
    }

    async fn get_by_id(&self, tenant_id: &str, id: Uuid) -> Outcome<Option<PersonalAccessToken>> {
        orm::Entity::find_by_id(id)
            .filter(orm::Column::TenantId.eq(tenant_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn get_batch(&self, tenant_id: &str, ids: &[Uuid]) -> Outcome<Vec<PersonalAccessToken>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let q = orm::Entity::find()
            .filter(orm::Column::Id.is_in(ids.to_vec()))
            .filter(orm::Column::TenantId.eq(tenant_id));
        q.all(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn get_by_hash(&self, token_hash: &str) -> Outcome<Option<PersonalAccessToken>> {
        orm::Entity::find()
            .filter(orm::Column::TokenHash.eq(token_hash))
            .one(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> Outcome<Vec<PersonalAccessToken>> {
        let models = orm::Entity::find()
            .filter(orm::Column::TenantId.eq(tenant_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?;

        models.into_iter().map(orm::Model::into_domain).collect()
    }

    async fn revoke(&self, tenant_id: Option<String>, id: Uuid) -> Outcome<()> {
        use sea_orm::sea_query::Expr;
        let res = orm::Entity::update_many()
            .col_expr(orm::Column::Revoked, Expr::value(true))
            .filter(orm::Column::Id.eq(id))
            .apply_if(tenant_id, |q, t| q.filter(orm::Column::TenantId.eq(t)))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?;

        if res.rows_affected == 0 {
            return Err(PatRepositoryError::NotFound.into_errors());
        }
        Ok(())
    }

    async fn update_last_used(&self, id: Uuid) -> Outcome<()> {
        let existing = orm::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?
            .ok_or_else(|| PatRepositoryError::NotFound.into_errors())?;

        let mut active: orm::ActiveModel = existing.into();
        active.last_used_at = Set(Some(Utc::now().into()));
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| PatRepositoryError::Db(Box::new(e)).into_errors())?;
        Ok(())
    }
}
