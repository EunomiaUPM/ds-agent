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

pub mod policy_templates;
pub mod types;
pub(crate) mod validator;

use crate::data::entities::policy_template;
use crate::data::entities::policy_template::{Model, NewPolicyTemplateModel};
use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::types::{LocalizedText, ParameterDefinition};
use common::dsp_common::odrl::OdrlPolicyInfo;
use common::paginated_spec::{Page, Paginated, Sort};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PolicyTemplateAllowedDefaultValues {
    Stringable(String),
    Numerable(f32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTemplateDto {
    pub id: String,
    pub tenant_id: String,
    pub version: String,
    pub date: DateTimeWithTimeZone,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub author: String,
    pub content: OdrlPolicyInfo,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyTemplateDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DateTimeWithTimeZone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub content: OdrlPolicyInfo,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,
}

use common::auth::AccessScope;

impl NewPolicyTemplateDto {
    pub fn into_model(self, tenant_id: String) -> Outcome<NewPolicyTemplateModel> {
        Ok(NewPolicyTemplateModel {
            id: self.id,
            tenant_id,
            version: self.version,
            date: self.date,
            author: self.author,
            title: self.title.map(serde_json::to_value).transpose()?,
            description: self.description.map(serde_json::to_value).transpose()?,
            content: serde_json::to_value(self.content)?,
            parameters: serde_json::to_value(self.parameters)?,
        })
    }
}

impl TryFrom<policy_template::Model> for PolicyTemplateDto {
    type Error = Errors;

    fn try_from(value: Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            tenant_id: value.tenant_id,
            version: value.version,
            date: value.date,
            title: value.title.map(serde_json::from_value).transpose()?,
            description: value.description.map(serde_json::from_value).transpose()?,
            author: value.author,
            content: serde_json::from_value(value.content)?,
            parameters: serde_json::from_value(value.parameters)?,
        })
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait PolicyTemplateEntityTrait: Sync + Send {
    async fn get_all_policy_templates(
        &self,
        scope: &AccessScope,
        filters: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PolicyTemplateDto>>;
    async fn get_batch_policy_templates(
        &self,
        scope: &AccessScope,
        ids: &[String],
    ) -> Outcome<Vec<PolicyTemplateDto>>;
    async fn get_policies_template_by_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
    ) -> Outcome<Vec<PolicyTemplateDto>>;
    async fn get_policies_template_by_version_and_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<PolicyTemplateDto>;
    async fn create_policy_template(
        &self,
        scope: &AccessScope,
        new_policy_template: &NewPolicyTemplateDto,
    ) -> Outcome<PolicyTemplateDto>;
    async fn delete_policy_template_by_version_and_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<()>;
}
