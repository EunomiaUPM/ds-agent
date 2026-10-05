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

//! Policy template repository.

use crate::data::entities::policy_template;
use common::oauth::OwnerScope;
use crate::data::entities::policy_template::NewPolicyTemplateModel;
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use crate::entities::filters::PolicyTemplateFilter;
use common::paginated_spec::{Page, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Persistence of policy templates, one row per template version.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PolicyTemplatesRepositoryTrait: Send + Sync {
    /// Page of templates matching the filters, with the total.
    async fn get_all_policy_templates(
        &self,
        scope: &OwnerScope,
        filters: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<policy_template::Model>, Option<u64>)>;
    async fn get_batch_policy_templates(
        &self,
        scope: &OwnerScope,
        ids: &[String],
    ) -> Outcome<Vec<policy_template::Model>>;
    /// Every version of the template.
    async fn get_policy_templates_by_id(
        &self,
        scope: &OwnerScope,
        template_id: &str,
    ) -> Outcome<Vec<policy_template::Model>>;
    async fn get_policy_template_by_id_and_version(
        &self,
        scope: &OwnerScope,
        template_id: &str,
        version: &str,
    ) -> Outcome<Option<policy_template::Model>>;
    async fn create_policy_template(
        &self,
        new_policy_template: &NewPolicyTemplateModel,
    ) -> Outcome<policy_template::Model>;
    async fn delete_policy_template_by_id_and_version(
        &self,
        scope: &OwnerScope,
        template_id: &str,
        version: &str,
    ) -> Outcome<()>;
}
