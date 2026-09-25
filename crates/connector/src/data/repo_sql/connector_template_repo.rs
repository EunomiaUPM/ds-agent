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
use crate::entities::filters::ConnectorTemplateFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<connector_templates::Entity>> for ConnectorTemplateFilter {
    fn apply_to(
        &self,
        mut select: Select<connector_templates::Entity>,
    ) -> Select<connector_templates::Entity> {
        if let Some(tenant_id) = &self.tenant_id {
            select = select.filter(connector_templates::Column::TenantId.eq(tenant_id));
        }
        if let Some(name) = &self.name {
            select = select.filter(connector_templates::Column::Name.eq(name));
        }
        if let Some(author) = &self.author {
            select = select.filter(connector_templates::Column::Author.eq(author));
        }
        if let Some(version) = &self.version {
            select = select.filter(connector_templates::Column::Version.eq(version));
        }
        if let Some(created_after) = self.created_after {
            select = select.filter(connector_templates::Column::CreatedAt.gte(created_after));
        }
        if let Some(created_before) = self.created_before {
            select = select.filter(connector_templates::Column::CreatedAt.lte(created_before));
        }
        select
    }
}

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
    #[tracing::instrument(level = "debug", skip_all, err)]
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

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_templates_by_name(
        &self,
        tenant_id: &str,
        template_name: &str,
    ) -> Outcome<Vec<connector_templates::Model>> {
        let result = connector_templates::Entity::find()
            .filter(connector_templates::Column::Name.eq(template_name))
            .filter(connector_templates::Column::TenantId.eq(tenant_id))
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

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_template_by_name_and_version(
        &self,
        tenant_id: &str,
        name: &str,
        version: &str,
    ) -> Outcome<Option<connector_templates::Model>> {
        let result =
            connector_templates::Entity::find_by_id((name.to_string(), version.to_string()))
                .filter(connector_templates::Column::TenantId.eq(tenant_id))
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

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_templates(
        &self,
        filters: &ConnectorTemplateFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<(Vec<connector_templates::Model>, Option<u64>)> {
        let q = filters.apply_to(connector_templates::Entity::find());
        let total = q.clone().count(&self.db_connection).await.map_err(|err| {
            ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                ConnectorTemplateRepoErrors::ErrorFetchingTemplate(err.to_string()),
            )
            .into_errors()
        })?;

        let list = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                connector_templates::Column::CreatedAt,
                connector_templates::Column::Name,
            )
            .all(&self.db_connection)
            .await
            .map_err(|err| {
                ConnectorAgentRepoErrors::ConnectorTemplateRepoErrors(
                    ConnectorTemplateRepoErrors::ErrorFetchingTemplate(err.to_string()),
                )
                .into_errors()
            })?;

        Ok((list, Some(total)))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_template_by_name_and_version(
        &self,
        tenant_id: &str,
        name: &str,
        version: &str,
    ) -> Outcome<()> {
        let result = connector_templates::Entity::delete_many()
            .filter(connector_templates::Column::Name.eq(name))
            .filter(connector_templates::Column::Version.eq(version))
            .filter(connector_templates::Column::TenantId.eq(tenant_id))
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
