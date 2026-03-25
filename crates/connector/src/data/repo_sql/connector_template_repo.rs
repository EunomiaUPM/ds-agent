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

use crate::data::entities::connector_templates;
use crate::data::entities::connector_templates::NewConnectorTemplateModel;
use crate::data::repo_traits::connector_repo_errors::{
    ConnectorAgentRepoErrors, ConnectorTemplateRepoErrors,
};
use crate::data::repo_traits::connector_template_repo::ConnectorTemplateRepoTrait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct ConnectorTemplateRepoForSql {
    db_connection: DatabaseConnection,
}

impl ConnectorTemplateRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl ConnectorTemplateRepoTrait for ConnectorTemplateRepoForSql {
    async fn create_template(
        &self,
        new_template_model: &NewConnectorTemplateModel,
    ) -> Outcome<connector_templates::Model> {
        let model: connector_templates::ActiveModel = new_template_model.clone().into();
        let template = connector_templates::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;

        match template {
            Ok(template) => Ok(template),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                ConnectorTemplateRepoErrors::ErrorCreatingTemplate(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn get_templates_by_name(
        &self,
        template_name: &String,
    ) -> Outcome<Vec<connector_templates::Model>> {
        let id_str = template_name.to_string();
        let result = connector_templates::Entity::find()
            .filter(connector_templates::Column::Name.eq(id_str.clone()))
            .all(&self.db_connection)
            .await;

        match result {
            Ok(opt) => Ok(opt),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                ConnectorTemplateRepoErrors::ErrorFetchingTemplate(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn get_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<Option<connector_templates::Model>> {
        let result = connector_templates::Entity::find_by_id((name.clone(), version.clone()))
            .one(&self.db_connection)
            .await;
        match result {
            Ok(opt) => Ok(opt),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                ConnectorTemplateRepoErrors::ErrorFetchingTemplate(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn get_all_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<connector_templates::Model>> {
        let page_limit = limit.unwrap_or(25);
        let page_number = page.unwrap_or(1);
        let calculated_offset = (page_number.max(1) - 1) * page_limit;
        let result = connector_templates::Entity::find()
            .limit(page_limit)
            .offset(calculated_offset)
            .all(&self.db_connection)
            .await;
        match result {
            Ok(list) => Ok(list),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                ConnectorTemplateRepoErrors::ErrorFetchingTemplate(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn delete_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<()> {
        let result = connector_templates::Entity::delete_by_id((name.clone(), version.clone()))
            .exec(&self.db_connection)
            .await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                    ConnectorTemplateRepoErrors::TemplateNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                ConnectorTemplateRepoErrors::ErrorDeletingTemplate(err.to_string()),
            )
            .into_errors()),
        }
    }
}
