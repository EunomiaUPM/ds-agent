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

use crate::data::entities::connector_templates;
use crate::data::entities::connector_templates::NewConnectorTemplateModel;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::{
    ConnectorMetadata, ConnectorTemplateDto, ConnectorTemplateEntitiesTrait,
};
use crate::entities::interaction::InteractionConfig;
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::template_parameters_extractor::TemplateParametersExtractor;
use crate::entities::parameters::template_parameters_validator::TemplateParametersValidator;
use crate::entities::parameters::ParameterDefinition;
use log::error;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct ConnectorTemplateEntitiesService {
    repo: Arc<dyn ConnectorRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl ConnectorTemplateEntitiesService {
    pub fn new(repo: Arc<dyn ConnectorRepoTrait>) -> Self {
        Self {
            repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }

    fn map_model_to_dto(model: connector_templates::Model) -> Outcome<ConnectorTemplateDto> {
        let spec = model.spec;
        let authentication: AuthenticationConfig = serde_json::from_value(
            spec.get("authentication")
                .ok_or_else(|| Errors::parse("Missing 'authentication' in template spec", None))?
                .clone(),
        )
        .map_err(|e| {
            Errors::parse(
                &format!("Error deserializing authentication config: {}", e),
                None,
            )
        })?;

        let interaction: InteractionConfig = serde_json::from_value(
            spec.get("interaction")
                .ok_or_else(|| Errors::parse("Missing 'interaction' in template spec", None))?
                .clone(),
        )
        .map_err(|e| {
            Errors::parse(
                &format!("Error deserializing interaction config: {}", e),
                None,
            )
        })?;

        let parameters: Vec<ParameterDefinition> = serde_json::from_value(
            spec.get("parameters")
                .ok_or_else(|| Errors::parse("Missing 'parameters' in template spec", None))?
                .clone(),
        )
        .map_err(|e| Errors::parse(&format!("Error deserializing parameters: {}", e), None))?;

        Ok(ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: Option::from(model.name),
                author: Option::from(model.author),
                description: None,
                version: Option::from(model.version),
                created_at: Some(model.created_at),
            },
            authentication,
            interaction,
            parameters,
        })
    }
}

use crate::entities::filters::ConnectorTemplateFilter;
use common::auth::AccessScope;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;

#[async_trait::async_trait]
impl ConnectorTemplateEntitiesTrait for ConnectorTemplateEntitiesService {
    async fn get_all_templates(
        &self,
        scope: &AccessScope,
        filters: &ConnectorTemplateFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<Paginated<ConnectorTemplateDto>> {
        scope.require_read()?;
        filters.validate()?;
        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;
        let page = page.clamped();

        let (models, total) = self
            .repo
            .get_templates_repo()
            .get_all_templates(&filters, &page, sort)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        let dtos: Outcome<Vec<ConnectorTemplateDto>> =
            models.into_iter().map(Self::map_model_to_dto).collect();
        let dtos = dtos?;

        Ok(Paginated::from_page(dtos, &page, total, |last| {
            Cursor::encode_composite(
                &last
                    .metadata
                    .created_at
                    .unwrap_or_else(|| chrono::Utc::now().into()),
                last.metadata.name.as_deref().unwrap_or_default(),
            )
        }))
    }

    async fn get_templates_by_id(
        &self,
        scope: &AccessScope,
        template_id: &str,
    ) -> Outcome<Vec<ConnectorTemplateDto>> {
        scope.require_read()?;
        let models = self
            .repo
            .get_templates_repo()
            .get_templates_by_name(scope.acting_tenant(), template_id)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        models.into_iter().map(Self::map_model_to_dto).collect()
    }

    async fn get_template_by_name_and_version(
        &self,
        scope: &AccessScope,
        name: &str,
        version: &str,
    ) -> Outcome<Option<ConnectorTemplateDto>> {
        scope.require_read()?;
        let result = self
            .repo
            .get_templates_repo()
            .get_template_by_name_and_version(scope.acting_tenant(), name, version)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        result.map(Self::map_model_to_dto).transpose()
    }

    async fn create_template(
        &self,
        scope: &AccessScope,
        new_template: &mut ConnectorTemplateDto,
    ) -> Outcome<ConnectorTemplateDto> {
        let target_tenant = scope.resolve_create_tenant(None)?;

        // extract parameters and validate
        let mut extractor = TemplateParametersExtractor::new();
        extractor.walk(new_template)?;
        let parameters_found = extractor.found_parameters();
        let validator =
            TemplateParametersValidator::new(parameters_found, &new_template.parameters)
                .excluding_sys_parameters()
                .excluding_runtime_parameters();
        validator.validate()?;

        // persist
        let new_model = new_template
            .clone()
            .into_model(target_tenant)
            .map_err(|e: Errors| {
                error!("{}", e);
                Errors::parse(&format!("Error preparing template model: {}", e), None)
            })?;
        let saved_model = self
            .repo
            .get_templates_repo()
            .create_template(&new_model)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;
        // create output
        let result = Self::map_model_to_dto(saved_model)?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "template",
            "create",
            &result
        );
        Ok(result)
    }

    async fn delete_template_by_name_and_version(
        &self,
        scope: &AccessScope,
        name: &str,
        version: &str,
    ) -> Outcome<()> {
        scope.require_write()?;

        self.repo
            .get_templates_repo()
            .delete_template_by_name_and_version(scope.acting_tenant(), name, version)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        let deleted = events::EntityDeletedDto::new(format!("{}:{}", name, version));
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "template",
            "delete",
            &deleted
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests;
