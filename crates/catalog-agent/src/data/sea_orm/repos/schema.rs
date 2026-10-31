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

//! Hosted schemas in Postgres.

use std::sync::Arc;

use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Select,
};
use ymir::errors::{Errors, Outcome, RepoIntoErrors};
use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Page, SelectCursorExt, Sort};

use crate::data::repo::schema::{SchemaRepoErrors, SchemaRepoTrait};
use crate::data::sea_orm::orm::schema as orm;
use crate::entities::commands::NewSchemaVersionCommand;
use crate::entities::filters::SchemaFilter;
use crate::entities::schemas::DataSchema;

pub struct SeaOrmSchemaRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmSchemaRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn scoped(scope: &OwnerScope) -> Select<orm::Entity> {
        orm::Entity::find().filter(scope.condition(
            orm::Column::UserId,
            orm::Column::UserRole,
            orm::Column::Visibility,
        ))
    }

    fn filtered(scope: &OwnerScope, filter: &SchemaFilter) -> Select<orm::Entity> {
        let mut q = Self::scoped(scope);
        if let Some(id) = &filter.id {
            q = q.filter(orm::Column::Id.eq(id.as_str()));
        }
        if let Some(title) = &filter.title {
            let escaped = title
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            q = q.filter(Expr::cust_with_values(
                "catalog_schemas.title ILIKE $1",
                [format!("%{escaped}%")],
            ));
        }
        if let Some(after) = filter.created_after {
            q = q.filter(orm::Column::CreatedAt.gte(after));
        }
        if let Some(before) = filter.created_before {
            q = q.filter(orm::Column::CreatedAt.lte(before));
        }
        q
    }

    fn fetch_err(e: DbErr) -> Errors {
        SchemaRepoErrors::Fetching(Box::new(e)).into_errors()
    }
}

#[async_trait::async_trait]
impl SchemaRepoTrait for SeaOrmSchemaRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &SchemaFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<DataSchema>> {
        let models = Self::filtered(scope, filter)
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                orm::Column::CreatedAt,
                orm::Column::Id,
            )
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Ok(models.into_iter().map(orm::Model::into_domain).collect())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn count(&self, scope: &OwnerScope, filter: &SchemaFilter) -> Outcome<u64> {
        Self::filtered(scope, filter)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_versions(&self, scope: &OwnerScope, id: &str) -> Outcome<Vec<DataSchema>> {
        let models = Self::scoped(scope)
            .filter(orm::Column::Id.eq(id))
            .order_by_desc(orm::Column::Version)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Ok(models.into_iter().map(orm::Model::into_domain).collect())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_version(
        &self,
        scope: &OwnerScope,
        id: &str,
        version: i32,
    ) -> Outcome<Option<DataSchema>> {
        Ok(Self::scoped(scope)
            .filter(orm::Column::Id.eq(id))
            .filter(orm::Column::Version.eq(version))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert(&self, command: &NewSchemaVersionCommand) -> Outcome<DataSchema> {
        Ok(orm::Entity::insert(orm::ActiveModel::from_command(command))
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(|e| SchemaRepoErrors::Creating(Box::new(e)).into_errors())?
            .into_domain())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete(&self, scope: &OwnerScope, id: &str, version: Option<i32>) -> Outcome<()> {
        let mut query = orm::Entity::delete_many()
            .filter(orm::Column::Id.eq(id))
            .filter(scope.condition(
                orm::Column::UserId,
                orm::Column::UserRole,
                orm::Column::Visibility,
            ));
        if let Some(version) = version {
            query = query.filter(orm::Column::Version.eq(version));
        }
        let deleted = query
            .exec(self.db.as_ref())
            .await
            .map_err(|e| SchemaRepoErrors::Deleting(Box::new(e)).into_errors())?;
        if deleted.rows_affected == 0 {
            return Err(SchemaRepoErrors::NotFound.into_errors());
        }
        Ok(())
    }
}
