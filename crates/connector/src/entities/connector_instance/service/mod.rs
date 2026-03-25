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

use crate::data::entities::connector_instances;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_instance::{
    ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto, InstanceMetadataDto,
};
use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use crate::entities::interaction::InteractionConfig;
use crate::facades::distribution_resolver_facade::DistributionFacadeTrait;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use crate::entities::parameters_ok::instance_parameters_map::InstanceParametersMapBuilder;
use crate::entities::parameters_ok::instance_parameters_resolver::InstanceParametersResolver;
use crate::entities::parameters_ok::instance_parameters_validator::InstanceParametersValidator;

pub struct ConnectorInstanceEntitiesService {
    repo: Arc<dyn ConnectorRepoTrait>,
    distribution_facade: Arc<dyn DistributionFacadeTrait>,
    /// The service's own base URL, used to resolve `{{__SYS_OWN_URL__}}` and
    /// `{{__SYS_OWN_URL_DOCKER__}}` placeholders during instance creation.
    own_url: String,
}

impl ConnectorInstanceEntitiesService {
    pub fn new(
        repo: Arc<dyn ConnectorRepoTrait>,
        distribution_facade: Arc<dyn DistributionFacadeTrait>,
        own_url: String,
    ) -> Self {
        Self {
            repo,
            distribution_facade,
            own_url,
        }
    }

    fn map_model_to_dto(model: connector_instances::Model) -> Outcome<ConnectorInstanceDto> {
        let auth_config: AuthenticationConfig =
            serde_json::from_value(model.authentication.clone())?;

        let interaction_config: InteractionConfig =
            serde_json::from_value(model.interaction.clone())?;

        let urn = Urn::from_str(&model.id)?;

        let distribution_urn = Urn::from_str(&model.distribution_id)?;

        let instance_meta: InstanceMetadataDto = serde_json::from_value(model.metadata.clone())
            .unwrap_or(InstanceMetadataDto {
                description: None,
                owner_id: None,
            });

        Ok(ConnectorInstanceDto {
            id: urn,
            metadata: ConnectorMetadata {
                name: Some(model.template_name),
                author: instance_meta.owner_id, // Not available in instance model
                description: instance_meta.description,
                version: Some(model.template_version), // available in instance model
                created_at: Some(model.created_at),
            },
            authentication_config: auth_config,
            interaction: interaction_config,
            distribution_id: distribution_urn,
        })
    }
}

#[async_trait::async_trait]
impl ConnectorInstanceTrait for ConnectorInstanceEntitiesService {
    async fn get_instance_by_id(&self, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>> {
        let id_str = id.to_string();
        let instance = self
            .repo
            .get_instances_repo()
            .get_instance_by_id(&id_str)
            .await?;

        match instance {
            Some(model) => Ok(Some(Self::map_model_to_dto(model)?)),
            None => Ok(None),
        }
    }

    async fn get_instance_by_distribution(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        let dist_id_str = distribution_id.to_string();

        let instance = self
            .repo
            .get_distro_relation_repo()
            .get_relation_by_distribution(&dist_id_str)
            .await?;

        if instance.is_none() {
            return Ok(None);
        }
        let instance = instance.unwrap();
        let result = self
            .repo
            .get_instances_repo()
            .get_instance_by_id(&instance.connector_instance_id)
            .await?;

        match result {
            Some(model) => Ok(Some(Self::map_model_to_dto(model)?)),
            None => Ok(None),
        }
    }

    async fn upsert_instance(
        &self,
        instance_dto: &mut ConnectorInstantiationDto,
    ) -> Outcome<ConnectorInstanceDto> {
        // fetch template or error
        let template_opt = self
            .repo
            .get_templates_repo()
            .get_template_by_name_and_version(
                &instance_dto.template_name,
                &instance_dto.template_version,
            )
            .await?;

        let template_model = match template_opt {
            Some(t) => t,
            None => {
                return Err(Errors::crazy(
                    format!(
                        "Template {} {} not found",
                        instance_dto.template_name, instance_dto.template_version
                    ),
                    None,
                ));
            }
        };
        let mut connector_template_dto: ConnectorTemplateDto =
            serde_json::from_value(template_model.spec.clone())?;

        // fetch distribution or error
        let distribution_id = instance_dto.distribution_id.to_string();
        let _ = self
            .distribution_facade
            .resolve_distribution_by_id(&distribution_id)
            .await?;

        // validate instance parameters
        let instance_parameters_validator = InstanceParametersValidator::new(
            &instance_dto.parameters,
            &connector_template_dto.parameters,
        );
        instance_parameters_validator.validate()?;

        // get all possible params (instance_dto + sys params + defaults)
        let params = InstanceParametersMapBuilder::new(self.own_url.clone())
            .with_instance_parameters(&instance_dto.parameters)
            .with_system_parameters(&mut connector_template_dto)?
            .with_default_parameters(&connector_template_dto.parameters)?
            .build()
            .collect();


        // resolve values in template
        let connector_resolved = InstanceParametersResolver::new(&connector_template_dto, &params)
            .resolve()?;

        // prepare data
        let metadata_json = connector_resolved.metadata.clone();
        let params_json = connector_resolved.parameters.clone();
        let authentication = connector_resolved.authentication.clone();
        let interaction = connector_resolved.interaction.clone();

        // dry run
        if instance_dto.dry_run {
            return Ok(ConnectorInstanceDto {
                id: Urn::from_str("urn:conector-instance:dry-run")?,
                metadata: metadata_json,
                authentication_config: authentication,
                interaction,
                distribution_id: instance_dto.distribution_id.clone(),
            });
        }

        // persist instance
        let new_instance = connector_instances::NewConnectorInstanceModel {
            id: None,
            template_name: instance_dto.template_name.clone(),
            template_version: instance_dto.template_version.clone(),
            distribution_id: distribution_id.clone(),
            metadata: serde_json::to_value(metadata_json)?,
            configuration_parameters: serde_json::to_value(params_json)?,
            authentication: serde_json::to_value(authentication)?,
            interaction: serde_json::to_value(interaction)?,
        };
        let saved_model = self
            .repo
            .get_instances_repo()
            .create_instance(&new_instance)
            .await?;

        // create or edit relation
        let instance_distro_relation = self
            .repo
            .get_distro_relation_repo()
            .get_relation_by_distribution(&distribution_id)
            .await?;
        match instance_distro_relation {
            None => {
                self.repo
                    .get_distro_relation_repo()
                    .create_relation(&distribution_id, &saved_model.id)
                    .await?
            }
            Some(_) => {
                self.repo
                    .get_distro_relation_repo()
                    .update_relation(&distribution_id, &saved_model.id)
                    .await?
            }
        };

        Self::map_model_to_dto(saved_model)
    }

    async fn delete_instance_by_id(&self, id: &Urn) -> Outcome<()> {
        let id_str = id.to_string();
        self.repo
            .get_instances_repo()
            .delete_instance_by_id(&id_str)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests;
