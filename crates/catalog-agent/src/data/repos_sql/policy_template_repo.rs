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

use crate::data::entities::policy_template;
use crate::data::entities::policy_template::{Model, NewPolicyTemplateModel};
use crate::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, OdrlOfferRepoErrors, PolicyTemplatesRepoErrors,
};
use crate::data::repo_traits::policy_template_repo::PolicyTemplatesRepositoryTrait;
use crate::entities::filters::PolicyTemplateFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<policy_template::Entity>> for PolicyTemplateFilter {
    fn apply_to(&self, mut q: Select<policy_template::Entity>) -> Select<policy_template::Entity> {
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(policy_template::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref id) = self.id {
            q = q.filter(policy_template::Column::Id.eq(id));
        }
        if let Some(ref version) = self.version {
            q = q.filter(policy_template::Column::Version.eq(version));
        }
        if let Some(ref author) = self.author {
            q = q.filter(policy_template::Column::Author.eq(author));
        }
        if let Some(after) = self.created_after {
            q = q.filter(policy_template::Column::Date.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(policy_template::Column::Date.lte(before));
        }
        q
    }
}

pub struct PolicyTemplatesRepositoryForSql {
    db_connection: DatabaseConnection,
}

impl PolicyTemplatesRepositoryForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl PolicyTemplatesRepositoryTrait for PolicyTemplatesRepositoryForSql {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_policy_templates(
        &self,
        filters: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<policy_template::Model>, Option<u64>)> {
        let mut q = policy_template::Entity::find();
        q = filters.apply_to(q);

        let total = q.clone().count(&self.db_connection).await.map_err(|err| {
            CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorFetchingPolicyTemplate(err.into()),
            )
            .into_errors()
        })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                policy_template::Column::Date,
                policy_template::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                    PolicyTemplatesRepoErrors::ErrorFetchingPolicyTemplate(err.into()),
                )
                .into_errors()
            })?;

        Ok((items, Some(total)))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch_policy_templates(
        &self,
        tenant_id: &str,
        ids: &[String],
    ) -> Outcome<Vec<policy_template::Model>> {
        let policy_ids = ids.to_vec();
        let policy_process = policy_template::Entity::find()
            .filter(policy_template::Column::TenantId.eq(tenant_id))
            .filter(policy_template::Column::Id.is_in(policy_ids))
            .all(&self.db_connection)
            .await;
        match policy_process {
            Ok(odrl_process) => Ok(odrl_process),
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorFetchingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_policy_templates_by_id(
        &self,
        tenant_id: &str,
        template_id: &str,
    ) -> Outcome<Vec<Model>> {
        match policy_template::Entity::find()
            .filter(policy_template::Column::TenantId.eq(tenant_id))
            .filter(policy_template::Column::Id.eq(template_id))
            .all(&self.db_connection)
            .await
        {
            Ok(template) => Ok(template),
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorFetchingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_policy_template_by_id_and_version(
        &self,
        tenant_id: &str,
        template_id: &str,
        version: &str,
    ) -> Outcome<Option<Model>> {
        match policy_template::Entity::find()
            .filter(policy_template::Column::TenantId.eq(tenant_id))
            .filter(policy_template::Column::Id.eq(template_id))
            .filter(policy_template::Column::Version.eq(version))
            .one(&self.db_connection)
            .await
        {
            Ok(template) => Ok(template),
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorFetchingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_policy_template(
        &self,
        new_policy_template: &NewPolicyTemplateModel,
    ) -> Outcome<policy_template::Model> {
        let model: policy_template::ActiveModel = new_policy_template.into();
        match policy_template::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await
        {
            Ok(template) => Ok(template),
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorCreatingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_policy_template_by_id_and_version(
        &self,
        tenant_id: &str,
        template_id: &str,
        version: &str,
    ) -> Outcome<()> {
        match policy_template::Entity::delete_many()
            .filter(policy_template::Column::TenantId.eq(tenant_id))
            .filter(policy_template::Column::Id.eq(template_id))
            .filter(policy_template::Column::Version.eq(version))
            .exec(&self.db_connection)
            .await
        {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                    PolicyTemplatesRepoErrors::PolicyTemplateNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorDeletingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }
}
