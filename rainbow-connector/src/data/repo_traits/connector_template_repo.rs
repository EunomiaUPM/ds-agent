/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::connector_templates;
use crate::data::entities::connector_templates::NewConnectorTemplateModel;
use crate::data::repo_traits::connector_repo_errors::ConnectorAgentRepoErrors;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorTemplateRepoTrait: Send + Sync {
    async fn create_template(
        &self,
        new_template_model: &NewConnectorTemplateModel,
    ) -> anyhow::Result<connector_templates::Model, ConnectorAgentRepoErrors>;

    async fn get_templates_by_name(
        &self,
        template_name: &String,
    ) -> anyhow::Result<Vec<connector_templates::Model>, ConnectorAgentRepoErrors>;

    async fn get_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> anyhow::Result<Option<connector_templates::Model>, ConnectorAgentRepoErrors>;

    async fn get_all_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<connector_templates::Model>, ConnectorAgentRepoErrors>;

    async fn delete_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> anyhow::Result<(), ConnectorAgentRepoErrors>;
}
