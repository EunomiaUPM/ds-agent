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

use crate::data::entities::connector_templates;
use crate::data::entities::connector_templates::NewConnectorTemplateModel;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::{
    ConnectorMetadata, ConnectorTemplateDto, ConnectorTemplateEntitiesTrait,
};
use crate::entities::interaction::InteractionConfig;
use crate::entities::parameters::parameters::ParameterDefinition;
use crate::entities::parameters::template_parameters_extractor::TemplateParameterExtractor;
use crate::entities::parameters::template_parameters_validator::ParameterValidator;
use crate::entities::parameters::template_parameters_visitor::ParameterExtractorVisitor;
use log::error;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct ConnectorTemplateEntitiesService {
    repo: Arc<dyn ConnectorRepoTrait>,
}

impl ConnectorTemplateEntitiesService {
    pub fn new(repo: Arc<dyn ConnectorRepoTrait>) -> Self {
        Self { repo }
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

#[async_trait::async_trait]
impl ConnectorTemplateEntitiesTrait for ConnectorTemplateEntitiesService {
    async fn get_all_templates(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<ConnectorTemplateDto>> {
        let models = self
            .repo
            .get_templates_repo()
            .get_all_templates(limit, page)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        let mut dtos = Vec::with_capacity(models.len());
        for model in models {
            dtos.push(Self::map_model_to_dto(model)?);
        }

        Ok(dtos)
    }

    async fn get_templates_by_id(
        &self,
        template_id: &String,
    ) -> Outcome<Vec<ConnectorTemplateDto>> {
        let models = self
            .repo
            .get_templates_repo()
            .get_templates_by_name(template_id)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        let mut dtos = Vec::with_capacity(models.len());
        for model in models {
            dtos.push(Self::map_model_to_dto(model)?);
        }
        Ok(dtos)
    }

    async fn get_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<Option<ConnectorTemplateDto>> {
        let result = self
            .repo
            .get_templates_repo()
            .get_template_by_name_and_version(name, version)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        match result {
            Some(model) => Ok(Some(Self::map_model_to_dto(model)?)),
            None => Ok(None),
        }
    }

    async fn create_template(
        &self,
        new_template: &mut ConnectorTemplateDto,
    ) -> Outcome<ConnectorTemplateDto> {
        // Validation: extract all {{__NAME__}} placeholders and check that
        // names and types match the declared parameters[]. RUNTIME_* / SYS_*
        // names are excluded because they are resolved by the engine at runtime.
        let mut extractor = TemplateParameterExtractor::new();
        ParameterExtractorVisitor::new(&mut extractor).extract(new_template);
        ParameterValidator::new(&new_template.parameters, true)
            .validate(extractor.found_parameters())
            .map_err(|e| {
                error!("{}", e);
                Errors::parse(&e.to_string(), None)
            })?;

        // persist
        let new_model: NewConnectorTemplateModel =
            new_template.clone().try_into().map_err(|e: Errors| {
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
        // create ouput
        Self::map_model_to_dto(saved_model)
    }

    async fn delete_template_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<()> {
        self.repo
            .get_templates_repo()
            .delete_template_by_name_and_version(name, version)
            .await
            .map_err(|e| {
                error!("{}", e);
                Errors::db(&e.to_string(), None)
            })?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests;
