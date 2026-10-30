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

//! Template service over the repositories. A template sees the pieces it references whoever owns
//! them: whoever stored it saw them, and referencing them shares them.

use std::collections::HashSet;
use std::sync::Arc;

use common::errors::NotFoundExt;
use common::oauth::{OwnedTrait, Owner, OwnerScope};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::entities::errors::CatalogErrors;
use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_composition::resolver::CompositionResolver;
use crate::entities::policy_composition::{TemplateLibrary, TemplatePreviewDto};
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateDto};
use crate::services::policy_templates::views::{PolicyPreviewView, TemplateUsagesView};
use crate::services::policy_templates::PolicyTemplateServiceTrait;
use crate::setup::context::AppContext;

pub struct PolicyTemplateService {
    ctx: Arc<AppContext>,
}

impl PolicyTemplateService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Loads the pieces below what the library holds, one query per level.
    async fn descend(&self, library: &mut TemplateLibrary, mut level: Vec<PolicyTemplateDto>) -> Outcome<()> {
        let repo = self.ctx.repos.policy_template_repo();
        let mut seen: HashSet<String> = HashSet::new();
        for _ in 0..=CompositionResolver::MAX_DEPTH {
            let mut keys = Vec::new();
            for template in &level {
                for (reference, _) in CompositionResolver::references(template.kind, &template.content)? {
                    if !library.contains(&reference) && seen.insert(reference.name()) {
                        keys.push((reference.id, reference.version));
                    }
                }
            }
            if keys.is_empty() {
                return Ok(());
            }
            level = repo.get_by_keys(&OwnerScope::All, &keys).await?;
            for template in &level {
                library.insert(template.clone());
            }
        }
        Ok(())
    }

    /// The draft as a template version, so the resolver can treat it like a stored one.
    fn draft(user: &UserInfo, draft: &NewPolicyTemplateDto) -> Outcome<PolicyTemplateDto> {
        draft.validate_dto()?;
        Ok(PolicyTemplateDto {
            id: draft.id.clone().unwrap_or_else(|| "draft".into()),
            user_id: user.id().to_string(),
            user_role: user.role().clone(),
            visibility: draft.visibility.clone().unwrap_or(ymir::types::participants::Visibility::Private),
            version: draft.version.clone().unwrap_or_else(|| "0".into()),
            kind: draft.kind,
            control_points: draft.control_points.clone(),
            date: chrono::Utc::now().into(),
            title: draft.title.clone(),
            description: draft.description.clone(),
            author: draft.author.clone().unwrap_or_default(),
            content: draft.stored_content()?,
            parameters: draft.parameters.clone(),
        })
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
        let mut command = template.into_command(owner)?;
        let references = CompositionResolver::references(command.kind, &command.content)?;
        let keys: Vec<(String, String)> =
            references.iter().map(|(r, _)| (r.id.clone(), r.version.clone())).collect();
        let library = self.library(user, keys.clone()).await?;
        let parameters = new_policy_template.parameters.clone();
        CompositionResolver::new(&library).check(command.kind, &command.content, &parameters)?;
        command.references = keys;
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
        let repo = self.ctx.repos.policy_template_repo();
        let dependents = repo.dependents(template_id, version_id).await?;
        if !dependents.is_empty() {
            return Err(CatalogErrors::conflict(format!(
                "{template_id}@{version_id} is referenced by {}; delete those first",
                dependents.join(", ")
            )));
        }
        repo.delete_version(&OwnerScope::acting(user), template_id, version_id).await?;
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn usages(
        &self,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) -> Outcome<TemplateUsagesView> {
        self.get_policies_template_by_version_and_id(user, template_id, version_id).await?;
        let templates = self.ctx.repos.policy_template_repo().dependents(template_id, version_id).await?;
        let offers = self
            .ctx
            .repos
            .odrl_offer_repo()
            .using_template(&OwnerScope::seeing(user), template_id, version_id)
            .await?;
        Ok(TemplateUsagesView { templates, offers: offers.iter().map(ToString::to_string).collect() })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn preview(&self, user: &UserInfo, request: &TemplatePreviewDto) -> Outcome<PolicyPreviewView> {
        let (template, mut library) = match (&request.template, &request.id, &request.version) {
            (Some(draft), None, None) => {
                let template = Self::draft(user, draft)?;
                let mut library = TemplateLibrary::default();
                self.descend(&mut library, vec![template.clone()]).await?;
                CompositionResolver::new(&library).check(template.kind, &template.content, &template.parameters)?;
                (template, library)
            }
            (None, Some(id), Some(version)) => {
                let library = self.library(user, vec![(id.clone(), version.clone())]).await?;
                let template = self.get_policies_template_by_version_and_id(user, id, version).await?;
                (template, library)
            }
            _ => {
                return Err(Errors::validation(
                    "give either id and version of a stored template, or a draft template",
                    None,
                ))
            }
        };
        library.insert(template.clone());
        let values = template.resolve_values(&request.values())?;
        let odrl = CompositionResolver::new(&library).expand(&template, &values)?;
        let control_points = library.control_points(&template).into_iter().collect();
        Ok(PolicyPreviewView { odrl, control_points })
    }

    #[tracing::instrument(level = "debug", skip_all, err, fields(user = %user.id()))]
    async fn library(&self, user: &UserInfo, keys: Vec<(String, String)>) -> Outcome<TemplateLibrary> {
        let roots = self
            .ctx
            .repos
            .policy_template_repo()
            .get_by_keys(&OwnerScope::seeing(user), &keys)
            .await?;
        let mut library = TemplateLibrary::new(roots.clone());
        self.descend(&mut library, roots).await?;
        Ok(library)
    }
}
