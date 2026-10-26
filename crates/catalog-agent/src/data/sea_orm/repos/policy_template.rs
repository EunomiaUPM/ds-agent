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

//! Policy templates in Postgres.

use std::sync::Arc;

use common::oauth::OwnerScope;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Select,
};
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

use crate::data::repo::policy_template::{PolicyTemplateRepoErrors, PolicyTemplateRepoTrait};
use crate::data::sea_orm::orm::policy_template as orm;
use crate::entities::commands::NewPolicyTemplateCommand;
use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::PolicyTemplateDto;

pub struct SeaOrmPolicyTemplateRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmPolicyTemplateRepo {
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

    fn filtered(scope: &OwnerScope, filter: &PolicyTemplateFilter) -> Select<orm::Entity> {
        let mut q = Self::scoped(scope);
        if let Some(user_id) = &filter.user_id {
            q = q.filter(orm::Column::UserId.eq(user_id.as_str()));
        }
        if let Some(id) = &filter.id {
            q = q.filter(orm::Column::Id.eq(id.as_str()));
        }
        if let Some(version) = &filter.version {
            q = q.filter(orm::Column::Version.eq(version.as_str()));
        }
        if let Some(author) = &filter.author {
            q = q.filter(orm::Column::Author.eq(author.as_str()));
        }
        if let Some(after) = filter.created_after {
            q = q.filter(orm::Column::Date.gte(after));
        }
        if let Some(before) = filter.created_before {
            q = q.filter(orm::Column::Date.lte(before));
        }
        q
    }

    fn decode(models: Vec<orm::Model>) -> Outcome<Vec<PolicyTemplateDto>> {
        models.into_iter().map(orm::Model::into_domain).collect()
    }

    fn fetch_err(e: DbErr) -> Errors {
        PolicyTemplateRepoErrors::Fetching(Box::new(e)).into_errors()
    }
}

#[async_trait::async_trait]
impl PolicyTemplateRepoTrait for SeaOrmPolicyTemplateRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        let models = Self::filtered(scope, filter)
            .apply_cursor_pagination_with_tie_break(page, sort, orm::Column::Date, orm::Column::Id)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn count(&self, scope: &OwnerScope, filter: &PolicyTemplateFilter) -> Outcome<u64> {
        Self::filtered(scope, filter)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch(&self, scope: &OwnerScope, ids: &[String]) -> Outcome<Vec<PolicyTemplateDto>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let models = Self::scoped(scope)
            .filter(orm::Column::Id.is_in(ids.to_vec()))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_versions(&self, scope: &OwnerScope, id: &str) -> Outcome<Vec<PolicyTemplateDto>> {
        let models = Self::scoped(scope)
            .filter(orm::Column::Id.eq(id))
            .order_by_desc(orm::Column::Date)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_version(
        &self,
        scope: &OwnerScope,
        id: &str,
        version: &str,
    ) -> Outcome<Option<PolicyTemplateDto>> {
        Self::scoped(scope)
            .filter(orm::Column::Id.eq(id))
            .filter(orm::Column::Version.eq(version))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert(&self, command: &NewPolicyTemplateCommand) -> Outcome<PolicyTemplateDto> {
        orm::Entity::insert(orm::ActiveModel::from_command(command))
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(|e| PolicyTemplateRepoErrors::Creating(Box::new(e)).into_errors())?
            .into_domain()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_version(&self, scope: &OwnerScope, id: &str, version: &str) -> Outcome<()> {
        let deleted = orm::Entity::delete_many()
            .filter(orm::Column::Id.eq(id))
            .filter(orm::Column::Version.eq(version))
            .filter(scope.condition(
                orm::Column::UserId,
                orm::Column::UserRole,
                orm::Column::Visibility,
            ))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| PolicyTemplateRepoErrors::Deleting(Box::new(e)).into_errors())?;
        if deleted.rows_affected == 0 {
            return Err(PolicyTemplateRepoErrors::NotFound.into_errors());
        }
        Ok(())
    }
}
