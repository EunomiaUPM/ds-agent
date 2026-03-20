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

use crate::data::entities::policy_template::NewPolicyTemplateModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::policy_templates::{
    NewPolicyTemplateDto, PolicyTemplateDto, PolicyTemplateEntityTrait,
};
use common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::Outcome;

pub struct PolicyTemplateEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
}

impl PolicyTemplateEntities {
    pub fn new(repo: Arc<dyn CatalogAgentRepoTrait>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl PolicyTemplateEntityTrait for PolicyTemplateEntities {
    async fn get_all_policy_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        let policy_templates = self
            .repo
            .get_policy_template_repo()
            .get_all_policy_templates(limit, page)
            .await?;
        let mut dtos = Vec::with_capacity(policy_templates.len());
        for c in policy_templates {
            let dto: PolicyTemplateDto = PolicyTemplateDto::try_from(c)?;
            dtos.push(dto);
        }
        Ok(dtos)
    }

    async fn get_batch_policy_templates(
        &self,
        ids: &Vec<String>,
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        let policy_templates =
            self.repo.get_policy_template_repo().get_batch_policy_templates(ids).await?;
        let mut dtos = Vec::with_capacity(policy_templates.len());
        for c in policy_templates {
            let dto: PolicyTemplateDto = PolicyTemplateDto::try_from(c)?;
            dtos.push(dto);
        }
        Ok(dtos)
    }

    async fn get_policies_template_by_id(
        &self,
        template_id: &String,
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        let policy_templates = self
            .repo
            .get_policy_template_repo()
            .get_policy_templates_by_id(template_id)
            .await?;
        let mut dtos = Vec::with_capacity(policy_templates.len());
        for c in policy_templates {
            let dto: PolicyTemplateDto = PolicyTemplateDto::try_from(c)?;
            dtos.push(dto);
        }
        Ok(dtos)
    }

    async fn get_policies_template_by_version_and_id(
        &self,
        template_id: &String,
        version_id: &String,
    ) -> Outcome<Option<PolicyTemplateDto>> {
        let policy_templates = self
            .repo
            .get_policy_template_repo()
            .get_policy_template_by_id_and_version(template_id, version_id)
            .await?;
        let dto: Option<PolicyTemplateDto> = policy_templates.map(TryInto::try_into).transpose()?;
        Ok(dto)
    }

    async fn create_policy_template(
        &self,
        new_policy_template: &NewPolicyTemplateDto,
    ) -> Outcome<PolicyTemplateDto> {
        new_policy_template.validate_dto()?;
        let new_model: NewPolicyTemplateModel = new_policy_template.clone().try_into()?;
        let policy_template =
            self.repo.get_policy_template_repo().create_policy_template(&new_model).await?;
        let dto: PolicyTemplateDto = PolicyTemplateDto::try_from(policy_template)?;
        Ok(dto)
    }

    async fn delete_policy_template_by_version_and_id(
        &self,
        template_id: &String,
        version_id: &String,
    ) -> Outcome<()> {
        let _ = self
            .repo
            .get_policy_template_repo()
            .delete_policy_template_by_id_and_version(template_id, version_id)
            .await?;
        Ok(())
    }
}
