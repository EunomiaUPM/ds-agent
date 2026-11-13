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
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use crate::entities::interaction::InteractionConfig;
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::template_parameters_extractor::TemplateParametersExtractor;
use crate::entities::parameters::template_parameters_validator::TemplateParametersValidator;
use crate::entities::parameters::ParameterDefinition;
use crate::services::connector_template::ConnectorTemplateServiceTrait;
use events::schemas::dataspace::{ConnectorLifecyclePayload, ResourceAction};
use events::{EventActor, EventContext, EventSinkTrait, NoopEventSink, TypedEventSink};
use log::error;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct ConnectorTemplateService {
    repo: Arc<dyn ConnectorRepoTrait>,
    events: Arc<dyn EventSinkTrait>,
}

impl ConnectorTemplateService {
    pub fn new(repo: Arc<dyn ConnectorRepoTrait>) -> Self {
        Self {
            repo,
            events: Arc::new(NoopEventSink),
        }
    }

    /// Publishes template lifecycle events on `events`.
    pub fn with_event_sink(mut self, events: Arc<dyn EventSinkTrait>) -> Self {
        self.events = events;
        self
    }

    async fn emit_lifecycle(
        &self,
        ctx: EventContext,
        action: ResourceAction,
        connector_id: String,
        name: String,
        connector_type: Option<String>,
    ) {
        let payload = ConnectorLifecyclePayload {
            action,
            connector_kind: "template".to_string(),
            connector_id,
            name,
            connector_type,
            status: None,
        };
        if let Err(err) = TypedEventSink::emit(&*self.events, &ctx, &payload).await {
            tracing::warn!(error = %err, "Failed to emit connector template lifecycle event");
        }
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
                format!("Error deserializing authentication config: {e}"),
                None,
            )
        })?;

        let interaction: InteractionConfig = serde_json::from_value(
            spec.get("interaction")
                .ok_or_else(|| Errors::parse("Missing 'interaction' in template spec", None))?
                .clone(),
        )
        .map_err(|e| Errors::parse(format!("Error deserializing interaction config: {e}"), None))?;

        let parameters: Vec<ParameterDefinition> = serde_json::from_value(
            spec.get("parameters")
                .ok_or_else(|| Errors::parse("Missing 'parameters' in template spec", None))?
                .clone(),
        )
        .map_err(|e| Errors::parse(format!("Error deserializing parameters: {e}"), None))?;

        let description = spec
            .get("description")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        Ok(ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: Option::from(model.name),
                author: Option::from(model.author),
                description,
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
use ymir::oauth::{OwnedTrait, Owner, OwnerScope};
use ymir::types::oauth::UserInfo;
use ymir::paginated_spec::{Cursor, Page, Paginated, Sort};
use ymir::query::QueryFilter;

#[async_trait::async_trait]
impl ConnectorTemplateServiceTrait for ConnectorTemplateService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_templates(
        &self,
        user: &UserInfo,
        filters: &ConnectorTemplateFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<Paginated<ConnectorTemplateDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (models, total) = self
            .repo
            .get_templates_repo()
            .get_all_templates(&OwnerScope::seeing(user), filters, &page, sort)
            .await
            .map_err(|e| {
                error!("{e}");
                Errors::db(e.to_string(), None)
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_templates_by_id(
        &self,
        user: &UserInfo,
        template_id: &str,
    ) -> Outcome<Vec<ConnectorTemplateDto>> {
        let models = self
            .repo
            .get_templates_repo()
            .get_templates_by_name(&OwnerScope::seeing(user), template_id)
            .await
            .map_err(|e| {
                error!("{e}");
                Errors::db(e.to_string(), None)
            })?;

        models.into_iter().map(Self::map_model_to_dto).collect()
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_template_by_name_and_version(
        &self,
        user: &UserInfo,
        name: &str,
        version: &str,
    ) -> Outcome<Option<ConnectorTemplateDto>> {
        let result = self
            .repo
            .get_templates_repo()
            .get_template_by_name_and_version(&OwnerScope::seeing(user), name, version)
            .await
            .map_err(|e| {
                error!("{e}");
                Errors::db(e.to_string(), None)
            })?;

        result.map(Self::map_model_to_dto).transpose()
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_template(
        &self,
        user: &UserInfo,
        new_template: &mut ConnectorTemplateDto,
    ) -> Outcome<ConnectorTemplateDto> {
        // extract parameters and validate
        new_template.validate_semantics()?;
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
            .into_model(Owner::private(user))
            .map_err(|e: Errors| {
                error!("{e}");
                Errors::parse(format!("Error preparing template model: {e}"), None)
            })?;
        let saved_model = self
            .repo
            .get_templates_repo()
            .create_template(&new_model)
            .await
            .map_err(|e| {
                error!("{e}");
                Errors::db(e.to_string(), None)
            })?;
        // create output
        let owner = saved_model.owner();
        let result = Self::map_model_to_dto(saved_model)?;
        let template_name = result.metadata.name.clone().unwrap_or_default();
        let template_version = result.metadata.version.clone().unwrap_or_default();
        let connector_type = match &result.interaction {
            InteractionConfig::Pull(_) => "pull",
            InteractionConfig::Push(_) => "push",
        };
        self.emit_lifecycle(
            EventContext::for_owner(owner).with_actor(EventActor::human(user)),
            ResourceAction::Created,
            format!("{template_name}:{template_version}"),
            template_name,
            Some(connector_type.to_string()),
        )
        .await;
        Ok(result)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_template_by_name_and_version(
        &self,
        user: &UserInfo,
        name: &str,
        version: &str,
    ) -> Outcome<()> {
        self.repo
            .get_templates_repo()
            .delete_template_by_name_and_version(&OwnerScope::acting(user), name, version)
            .await?;

        self.emit_lifecycle(
            EventContext::for_user(user),
            ResourceAction::Deleted,
            format!("{name}:{version}"),
            name.to_string(),
            None,
        )
        .await;

        Ok(())
    }
}
