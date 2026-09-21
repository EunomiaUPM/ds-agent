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

use crate::data::entities::policy_template::NewPolicyTemplateModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::{
    NewPolicyTemplateDto, PolicyTemplateDto, PolicyTemplateEntityTrait,
};
use common::auth::AccessScope;
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct PolicyTemplateEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl PolicyTemplateEntities {
    pub fn new(repo: Arc<dyn CatalogAgentRepoTrait>) -> Self {
        Self {
            repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl PolicyTemplateEntityTrait for PolicyTemplateEntities {
    async fn get_all_policy_templates(
        &self,
        scope: &AccessScope,
        filters: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PolicyTemplateDto>> {
        scope.require_read()?;
        filters.validate()?;
        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;
        let page = page.clamped();

        let (policy_templates, total) = self
            .repo
            .get_policy_template_repo()
            .get_all_policy_templates(&filters, &page, sort)
            .await?;

        let dtos = policy_templates
            .into_iter()
            .map(PolicyTemplateDto::try_from)
            .collect::<Outcome<Vec<_>>>()?;

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.date, &d.id)
        }))
    }

    async fn get_batch_policy_templates(
        &self,
        scope: &AccessScope,
        ids: &[String],
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        scope.require_read()?;
        let policy_templates = self
            .repo
            .get_policy_template_repo()
            .get_batch_policy_templates(scope.acting_tenant(), ids)
            .await?;
        policy_templates
            .into_iter()
            .map(PolicyTemplateDto::try_from)
            .collect()
    }

    async fn get_policies_template_by_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        scope.require_read()?;
        let policy_templates = self
            .repo
            .get_policy_template_repo()
            .get_policy_templates_by_id(scope.acting_tenant(), template_id)
            .await?;
        policy_templates
            .into_iter()
            .map(PolicyTemplateDto::try_from)
            .collect()
    }

    async fn get_policies_template_by_version_and_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<PolicyTemplateDto> {
        scope.require_read()?;
        let policy_template = self
            .repo
            .get_policy_template_repo()
            .get_policy_template_by_id_and_version(scope.acting_tenant(), template_id, version_id)
            .await?
            .or_not_found(format!("{template_id}:{version_id}"), "policy template")?;
        PolicyTemplateDto::try_from(policy_template)
    }

    async fn create_policy_template(
        &self,
        scope: &AccessScope,
        new_policy_template: &NewPolicyTemplateDto,
    ) -> Outcome<PolicyTemplateDto> {
        new_policy_template.validate_dto()?;
        let mut new_policy_template = new_policy_template.clone();
        let tenant_id = scope.resolve_create_tenant(new_policy_template.tenant_id.as_deref())?;
        new_policy_template.tenant_id = Some(tenant_id.clone());
        let new_model: NewPolicyTemplateModel = new_policy_template.into_model(tenant_id)?;
        let policy_template = self
            .repo
            .get_policy_template_repo()
            .create_policy_template(&new_model)
            .await?;
        let dto: PolicyTemplateDto = PolicyTemplateDto::try_from(policy_template)?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "policy_template",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn delete_policy_template_by_version_and_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<()> {
        scope.require_write()?;
        self.repo
            .get_policy_template_repo()
            .delete_policy_template_by_id_and_version(
                scope.acting_tenant(),
                template_id,
                version_id,
            )
            .await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "policy_template",
            "delete",
            &events::EntityDeletedDto::new(format!("{template_id}:{version_id}"))
        );
        Ok(())
    }
}
