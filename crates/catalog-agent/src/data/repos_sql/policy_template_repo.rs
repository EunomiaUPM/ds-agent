/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

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
    async fn get_all_policy_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<policy_template::Model>> {
        let page_limit = limit.unwrap_or(25);
        let page_number = page.unwrap_or(1);
        let calculated_offset = (page_number.max(1) - 1) * page_limit;

        match policy_template::Entity::find()
            .order_by_desc(policy_template::Column::Date)
            .limit(page_limit)
            .offset(calculated_offset)
            .all(&self.db_connection)
            .await
        {
            Ok(templates) => Ok(templates),
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorFetchingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_batch_policy_templates(
        &self,
        ids: &Vec<String>,
    ) -> Outcome<Vec<policy_template::Model>> {
        let policy_ids = ids.clone();
        let policy_process = policy_template::Entity::find()
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

    async fn get_policy_templates_by_id(&self, template_id: &String) -> Outcome<Vec<Model>> {
        let template_id = template_id.to_string();
        match policy_template::Entity::find()
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

    async fn get_policy_template_by_id_and_version(
        &self,
        template_id: &String,
        version: &String,
    ) -> Outcome<Option<Model>> {
        match policy_template::Entity::find_by_id((template_id.clone(), version.clone()))
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

    async fn create_policy_template(
        &self,
        new_policy_template: &NewPolicyTemplateModel,
    ) -> Outcome<policy_template::Model> {
        let model: policy_template::ActiveModel = new_policy_template.into();
        match policy_template::Entity::insert(model).exec_with_returning(&self.db_connection).await
        {
            Ok(template) => Ok(template),
            Err(err) => Err(CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
                PolicyTemplatesRepoErrors::ErrorCreatingPolicyTemplate(err.into()),
            )
            .into_errors()),
        }
    }

    async fn delete_policy_template_by_id_and_version(
        &self,
        template_id: &String,
        version: &String,
    ) -> Outcome<()> {
        match policy_template::Entity::delete_by_id((template_id.clone(), version.clone()))
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
