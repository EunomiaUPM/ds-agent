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
use crate::data::entities::policy_template::NewPolicyTemplateModel;
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait PolicyTemplatesRepositoryTrait: Send + Sync {
    async fn get_all_policy_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<policy_template::Model>>;
    async fn get_batch_policy_templates(
        &self,
        ids: &Vec<String>,
    ) -> Outcome<Vec<policy_template::Model>>;
    async fn get_policy_templates_by_id(
        &self,
        template_id: &String,
    ) -> Outcome<Vec<policy_template::Model>>;
    async fn get_policy_template_by_id_and_version(
        &self,
        template_id: &String,
        version: &String,
    ) -> Outcome<Option<policy_template::Model>>;
    async fn create_policy_template(
        &self,
        new_policy_template: &NewPolicyTemplateModel,
    ) -> Outcome<policy_template::Model>;
    async fn delete_policy_template_by_id_and_version(
        &self,
        template_id: &String,
        version: &String,
    ) -> Outcome<()>;
}
