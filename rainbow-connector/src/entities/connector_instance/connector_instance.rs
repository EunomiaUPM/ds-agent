use crate::data::entities::connector_instances;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::common::default_parameter::ParameterDefaultInjector;
use crate::entities::common::parameters::TemplateMutable;
use crate::entities::common::system_context::SystemContext;
use crate::entities::common::system_parameter::SystemParameterInjector;
use crate::entities::connector_instance::parameter_validator::InstanceParameterValidator;
use crate::entities::connector_instance::resolver::TemplateResolver;
use crate::entities::connector_instance::{
    ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto, InstanceMetadataDto,
};
use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use crate::entities::interaction::InteractionConfig;
use crate::facades::distribution_resolver_facade::DistributionFacadeTrait;
use anyhow::{anyhow, bail};
use log::error;
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use urn::{Urn, UrnBuilder};

pub struct ConnectorInstanceEntitiesService {
    repo: Arc<dyn ConnectorRepoTrait>,
    distribution_facade: Arc<dyn DistributionFacadeTrait>,
}

impl ConnectorInstanceEntitiesService {
    pub fn new(
        repo: Arc<dyn ConnectorRepoTrait>,
        distribution_facade: Arc<dyn DistributionFacadeTrait>,
    ) -> Self {
        Self { repo, distribution_facade }
    }

    fn map_model_to_dto(model: connector_instances::Model) -> anyhow::Result<ConnectorInstanceDto> {
        let auth_config: AuthenticationConfig =
            serde_json::from_value(model.authentication.clone()).map_err(|e| {
                let err = CommonErrors::parse_new(&format!(
                    "Error deserializing authentication config: {}",
                    e
                ));
                error!("{}", err.log());
                err
            })?;

        let interaction_config: InteractionConfig =
            serde_json::from_value(model.interaction.clone()).map_err(|e| {
                let err = CommonErrors::parse_new(&format!(
                    "Error deserializing interaction config: {}",
                    e
                ));
                error!("{}", err.log());
                err
            })?;

        let urn = Urn::from_str(&model.id).map_err(|e| {
            let err = CommonErrors::parse_new(&format!("Error parsing URN: {}", e));
            error!("{}", err.log());
            err
        })?;

        let distribution_urn = Urn::from_str(&model.distribution_id).map_err(|e| {
            let err = CommonErrors::parse_new(&format!("Error parsing Distribution URN: {}", e));
            error!("{}", err.log());
            err
        })?;

        let instance_meta: InstanceMetadataDto = serde_json::from_value(model.metadata.clone())
            .unwrap_or(InstanceMetadataDto { description: None, owner_id: None });

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
    async fn get_instance_by_id(&self, id: &Urn) -> anyhow::Result<Option<ConnectorInstanceDto>> {
        let id_str = id.to_string();
        let instance =
            self.repo.get_instances_repo().get_instance_by_id(&id_str).await.map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        match instance {
            Some(model) => Ok(Some(Self::map_model_to_dto(model)?)),
            None => Ok(None),
        }
    }

    async fn get_instance_by_distribution(
        &self,
        distribution_id: &Urn,
    ) -> anyhow::Result<Option<ConnectorInstanceDto>> {
        let dist_id_str = distribution_id.to_string();

        let instance = self
            .repo
            .get_distro_relation_repo()
            .get_relation_by_distribution(&dist_id_str)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        if instance.is_none() {
            return Ok(None);
        }
        let instance = instance.unwrap();
        let result = self
            .repo
            .get_instances_repo()
            .get_instance_by_id(&instance.connector_instance_id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        match result {
            Some(model) => Ok(Some(Self::map_model_to_dto(model)?)),
            None => Ok(None),
        }
    }

    async fn upsert_instance(
        &self,
        instance_dto: &mut ConnectorInstantiationDto,
    ) -> anyhow::Result<ConnectorInstanceDto> {
        // fetch template or error
        let template = self
            .repo
            .get_templates_repo()
            .get_template_by_name_and_version(
                &instance_dto.template_name,
                &instance_dto.template_version,
            )
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        let template_model = match template {
            Some(t) => t,
            None => {
                let err = CommonErrors::missing_resource_new(
                    "template",
                    &format!(
                        "Template {} {} not found",
                        instance_dto.template_name, instance_dto.template_version
                    ),
                );
                error!("{}", err.log());
                return Err(anyhow::anyhow!(err));
            }
        };

        // fetch distribution or error
        let distribution_id = instance_dto.distribution_id.to_string();
        let _ =
            self.distribution_facade.resolve_distribution_by_id(&distribution_id).await.map_err(
                |e| {
                    let err = CommonErrors::parse_new(&format!(
                        "Error resolving associated distribution: {}",
                        e
                    ));
                    error!("{}", err.log());
                    err
                },
            )?;

        // validate parameters
        let mut template_spec: ConnectorTemplateDto =
            serde_json::from_value(template_model.spec.clone())?;
        let template_parameters = &template_spec.parameters;
        let validation_errors =
            InstanceParameterValidator::validate(template_parameters, &instance_dto.parameters);
        if !validation_errors.is_empty() {
            let err = CommonErrors::parse_new(&format!("{}", validation_errors.join(", ")));
            error!("{}", err.log());
            bail!(err);
        }

        // inject sys parameters to be interpolated
        let context = SystemContext::new();
        SystemParameterInjector::inject(
            &template_spec.parameters,
            &mut instance_dto.parameters,
            &context,
        );

        // apply defaults
        ParameterDefaultInjector::inject(&template_spec.parameters, &mut instance_dto.parameters)
            .map_err(|e| {
            let err = CommonErrors::parse_new(&format!("{}", validation_errors.join(", ")));
            error!("{}", err.log());
            anyhow!(err)
        })?;

        // interpolate values
        let mut resolver = TemplateResolver::new(&instance_dto.parameters);
        template_spec.interaction.accept_mutator(&mut resolver)?;
        template_spec.authentication.accept_mutator(&mut resolver)?;

        // prepare data
        let metadata_json = template_spec.metadata.clone();
        let params_json = template_spec.parameters.clone();
        let authentication = template_spec.authentication.clone();
        let interaction = template_spec.interaction.clone();

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
        let saved_model =
            self.repo.get_instances_repo().create_instance(&new_instance).await.map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        // create or edit relation
        let instance_distro_relation = self
            .repo
            .get_distro_relation_repo()
            .get_relation_by_distribution(&distribution_id)
            .await
            .map_err(|e| {
                let err = CommonErrors::parse_new(&format!("Db error on getting relation: {}", e));
                error!("{}", err.log());
                err
            })?;
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

    async fn delete_instance_by_id(&self, id: &Urn) -> anyhow::Result<()> {
        let id_str = id.to_string();
        self.repo.get_instances_repo().delete_instance_by_id(&id_str).await.map_err(|e| {
            let err = CommonErrors::database_new(&e.to_string());
            error!("{}", err.log());
            err
        })?;
        Ok(())
    }
}
