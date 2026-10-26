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

//! Template service over the repository.

use std::sync::Arc;

use common::errors::NotFoundExt;
use common::oauth::{OwnedTrait, Owner, OwnerScope};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateDto};
use crate::services::policy_templates::PolicyTemplateServiceTrait;
use crate::setup::context::AppContext;

pub struct PolicyTemplateService {
    ctx: Arc<AppContext>,
}

impl PolicyTemplateService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl PolicyTemplateServiceTrait for PolicyTemplateService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_policy_templates(
        &self,
        user: &UserInfo,
        filters: &PolicyTemplateFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PolicyTemplateDto>> {
        filters.validate()?;
        let page = page.clamped();
        let scope = OwnerScope::seeing(user);
        let repo = self.ctx.repos.policy_template_repo();
        let items = repo.get_page(&scope, filters, &page, sort).await?;
        let total = repo.count(&scope, filters).await?;
        Ok(Paginated::from_page(items, &page, Some(total), |d| {
            Cursor::encode_composite(&d.date, &d.id)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_batch_policy_templates(
        &self,
        user: &UserInfo,
        ids: &[String],
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        self.ctx
            .repos
            .policy_template_repo()
            .get_batch(&OwnerScope::seeing(user), ids)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_policies_template_by_id(
        &self,
        user: &UserInfo,
        template_id: &str,
    ) -> Outcome<Vec<PolicyTemplateDto>> {
        self.ctx
            .repos
            .policy_template_repo()
            .get_versions(&OwnerScope::seeing(user), template_id)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_policies_template_by_version_and_id(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<PolicyTemplateDto> {
        self.ctx
            .repos
            .policy_template_repo()
            .get_version(&OwnerScope::seeing(user), template_id, version_id)
            .await?
            .or_not_found(format!("{template_id}:{version_id}"), "policy template")
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_policy_template(
        &self,
        user: &UserInfo,
        new_policy_template: &NewPolicyTemplateDto,
    ) -> Outcome<PolicyTemplateDto> {
        new_policy_template.validate_dto()?;
        let mut template = new_policy_template.clone();
        let owner = Owner::for_new(user, template.owner.take(), template.visibility.clone());
        let command = template.into_command(owner)?;
        let stored = self.ctx.repos.policy_template_repo().insert(&command).await?;
        events::emit_action!(
            self.ctx.events,
            &stored.owner(),
            crate::EVENT_PREFIX,
            "policy_template",
            "create",
            &stored
        );
        Ok(stored)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_policy_template_by_version_and_id(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<()> {
        self.ctx
            .repos
            .policy_template_repo()
            .delete_version(&OwnerScope::acting(user), template_id, version_id)
            .await?;
        events::emit_action!(
            self.ctx.events,
            &Owner::private(user),
            crate::EVENT_PREFIX,
            "policy_template",
            "delete",
            &events::EntityDeletedDto::new(format!("{template_id}:{version_id}"))
        );
        Ok(())
    }
}
