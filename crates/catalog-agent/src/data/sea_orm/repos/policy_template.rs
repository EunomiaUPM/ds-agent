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

//! Policy templates in Postgres; a version and its references are written in one transaction.

use std::sync::Arc;

use common::oauth::OwnerScope;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Select, TransactionTrait,
};
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

use crate::data::repo::policy_template::{PolicyTemplateRepoErrors, PolicyTemplateRepoTrait};
use crate::data::sea_orm::orm::policy_template as orm;
use crate::data::sea_orm::orm::policy_template_ref as refs;
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
        if let Some(kind) = filter.kind {
            q = q.filter(orm::Column::Kind.eq(kind.as_str()));
        }
        if let Some(point) = filter.control_point {
            q = q.filter(Expr::cust_with_values(
                "catalog_policy_templates.control_points @> $1",
                [serde_json::json!([point])],
            ));
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
    async fn get_by_keys(
        &self,
        scope: &OwnerScope,
        keys: &[(String, String)],
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let any = keys.iter().fold(Condition::any(), |any, (id, version)| {
            any.add(
                Condition::all()
                    .add(orm::Column::Id.eq(id.as_str()))
                    .add(orm::Column::Version.eq(version.as_str())),
            )
        });
        let models = Self::scoped(scope)
            .filter(any)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Self::decode(models)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert(&self, command: &NewPolicyTemplateCommand) -> Outcome<PolicyTemplateDto> {
        let create_err = |e: DbErr| PolicyTemplateRepoErrors::Creating(Box::new(e)).into_errors();
        let txn = self.db.begin().await.map_err(create_err)?;
        let stored = orm::Entity::insert(orm::ActiveModel::from_command(command))
            .exec_with_returning(&txn)
            .await
            .map_err(create_err)?
            .into_domain()?;
        if !command.references.is_empty() {
            let rows = command.references.iter().map(|(id, version)| refs::ActiveModel {
                template_id: sea_orm::ActiveValue::Set(stored.id.clone()),
                template_version: sea_orm::ActiveValue::Set(stored.version.clone()),
                ref_id: sea_orm::ActiveValue::Set(id.clone()),
                ref_version: sea_orm::ActiveValue::Set(version.clone()),
            });
            refs::Entity::insert_many(rows)
                .on_conflict_do_nothing()
                .exec(&txn)
                .await
                .map_err(create_err)?;
        }
        txn.commit().await.map_err(create_err)?;
        Ok(stored)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn dependents(&self, id: &str, version: &str) -> Outcome<Vec<String>> {
        let rows = refs::Entity::find()
            .filter(refs::Column::RefId.eq(id))
            .filter(refs::Column::RefVersion.eq(version))
            .order_by_asc(refs::Column::TemplateId)
            .order_by_asc(refs::Column::TemplateVersion)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;
        Ok(rows
            .into_iter()
            .map(|r| format!("{}@{}", r.template_id, r.template_version))
            .collect())
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
