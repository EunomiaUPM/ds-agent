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

//! Policy template management use cases: versions, references between them and previews.

pub mod service;
pub mod views;

use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_composition::{TemplateLibrary, TemplatePreviewDto};
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateDto};
use crate::services::policy_templates::views::{PolicyPreviewView, TemplateUsagesView};
use ymir::paginated_spec::{Page, Paginated, Sort};
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

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
    /// Validates the template and its references and stores it with them.
    async fn create_policy_template(
        &self,
        user: &UserInfo,
        new_policy_template: &NewPolicyTemplateDto,
    ) -> Outcome<PolicyTemplateDto>;
    /// Fails with a 409 while another template version references it.
    async fn delete_policy_template_by_version_and_id(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<()>;
    /// Template versions and offers built from this version.
    async fn usages(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<TemplateUsagesView>;
    /// The plain ODRL of a stored version or a draft, filled with the given values; stores nothing.
    async fn preview(&self, user: &UserInfo, request: &TemplatePreviewDto) -> Outcome<PolicyPreviewView>;
    /// The versions named by `(id, version)` that the caller sees, and every piece below them.
    async fn library(&self, user: &UserInfo, keys: Vec<(String, String)>) -> Outcome<TemplateLibrary>;
}
