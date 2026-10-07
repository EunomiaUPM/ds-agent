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

//! Policy template management use cases.

pub mod service;

use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateDto};
use common::oauth::UserInfo;
use common::paginated_spec::{Page, Paginated, Sort};
use ymir::errors::Outcome;

/// Management of policy templates and their versions.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PolicyTemplateServiceTrait: Sync + Send {
    /// Page of templates visible to the caller.
    async fn get_all_policy_templates(
        &self,
        user: &UserInfo,
        filters: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PolicyTemplateDto>>;
    /// Templates found among `ids`.
    async fn get_batch_policy_templates(
        &self,
        user: &UserInfo,
        ids: &[String],
    ) -> Outcome<Vec<PolicyTemplateDto>>;
    /// Every version of the template.
    async fn get_policies_template_by_id(
        &self,
        user: &UserInfo,
        template_id: &str,
    ) -> Outcome<Vec<PolicyTemplateDto>>;
    async fn get_policies_template_by_version_and_id(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<PolicyTemplateDto>;
    /// Validates the template and stores it.
    async fn create_policy_template(
        &self,
        user: &UserInfo,
        new_policy_template: &NewPolicyTemplateDto,
    ) -> Outcome<PolicyTemplateDto>;
    async fn delete_policy_template_by_version_and_id(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<()>;
}
