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

use crate::data::entities::connector_instances;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_instance::{
    ConnectorInstanceDto, ConnectorInstantiationDto, InstanceMetadataDto,
};
use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use crate::entities::errors::ConnectorErrors;
use crate::entities::interaction::InteractionConfig;
use crate::entities::parameters::instance_parameters_map::InstanceParametersMapBuilder;
use crate::entities::parameters::instance_parameters_resolver::InstanceParametersResolver;
use crate::entities::parameters::instance_parameters_validator::InstanceParametersValidator;
use crate::services::connector_instance::ConnectorInstanceServiceTrait;
use crate::setup::ActiveTransferGuard;
use common::facades::catalog::CatalogFacadeTrait;
use events::schemas::dataspace::{
    ConnectorLifecyclePayload, PolicyDecisionOutcome, PolicyDecisionPayload, ResourceAction,
};
use events::{EventActor, EventContext, EventSinkTrait, NoopEventSink, TypedEventSink};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[derive(Default)]
struct InstanceL1Cache {
    by_id: HashMap<(String, String, String), (Instant, ConnectorInstanceDto)>,
    by_distro: HashMap<(String, String, String), (Instant, ConnectorInstanceDto)>,
}

pub struct ConnectorInstanceService {
    repo: Arc<dyn ConnectorRepoTrait>,
    distribution_facade: Arc<dyn CatalogFacadeTrait>,
    /// The service's own base URL, used to resolve `{{__SYS_OWN_URL__}}` and
    /// `{{__SYS_OWN_URL_DOCKER__}}` placeholders during instance creation.
    own_url: String,
    events: Arc<dyn EventSinkTrait>,
    transfer_guard: Option<Arc<dyn ActiveTransferGuard>>,
    l1: RwLock<InstanceL1Cache>,
}

impl ConnectorInstanceService {
    const L1_TTL: Duration = Duration::from_secs(5);
    const L1_MAX_ENTRIES: usize = 2048;

    pub fn new(
        repo: Arc<dyn ConnectorRepoTrait>,
        distribution_facade: Arc<dyn CatalogFacadeTrait>,
        own_url: String,
    ) -> Self {
        Self {
            repo,
            distribution_facade,
            own_url,
            events: Arc::new(NoopEventSink),
            transfer_guard: None,
            l1: RwLock::new(InstanceL1Cache::default()),
        }
    }

    fn cache_key(user: &UserInfo, id: &str) -> (String, String, String) {
        (user.id().to_string(), user.role().to_string(), id.to_string())
    }

    fn read_by_id(&self, user: &UserInfo, id: &str) -> Option<ConnectorInstanceDto> {
        let key = Self::cache_key(user, id);
        let guard = self.l1.read().ok()?;
        let (ts, dto) = guard.by_id.get(&key)?;
        (ts.elapsed() < Self::L1_TTL).then(|| dto.clone())
    }

    fn read_by_distro(&self, user: &UserInfo, distro_id: &str) -> Option<ConnectorInstanceDto> {
        let key = Self::cache_key(user, distro_id);
        let guard = self.l1.read().ok()?;
        let (ts, dto) = guard.by_distro.get(&key)?;
        (ts.elapsed() < Self::L1_TTL).then(|| dto.clone())
    }

    fn write_l1(
        &self,
        user: &UserInfo,
        distro_id: Option<&str>,
        dto: &ConnectorInstanceDto,
    ) {
        if let Ok(mut guard) = self.l1.write() {
            let now = Instant::now();
            if guard.by_id.len() >= Self::L1_MAX_ENTRIES {
                guard.by_id.retain(|_, (ts, _)| ts.elapsed() < Self::L1_TTL);
            }
            if guard.by_distro.len() >= Self::L1_MAX_ENTRIES {
                guard.by_distro.retain(|_, (ts, _)| ts.elapsed() < Self::L1_TTL);
            }
            guard.by_id.insert(
                Self::cache_key(user, &dto.id.to_string()),
                (now, dto.clone()),
            );
            if let Some(d) = distro_id {
                guard
                    .by_distro
                    .insert(Self::cache_key(user, d), (now, dto.clone()));
            }
        }
    }

    fn evict_instance(&self, id_str: &str, distro_id: Option<&str>) {
        if let Ok(mut guard) = self.l1.write() {
            guard.by_id.retain(|(_, _, k), _| k != id_str);
            guard.by_distro.retain(|(_, _, d), (_, dto)| {
                dto.id.to_string() != id_str && distro_id != Some(d.as_str())
            });
        }
    }

    /// Publishes instance lifecycle events on `events`.
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
            connector_kind: "instance".to_string(),
            connector_id,
            name,
            connector_type,
            status: None,
        };
        if let Err(err) = TypedEventSink::emit(&*self.events, &ctx, &payload).await {
            tracing::warn!(error = %err, "Failed to emit connector instance lifecycle event");
        }
    }

    async fn emit_guard_decision(
        &self,
        ctx: EventContext,
        decision: PolicyDecisionOutcome,
        connector_id: &str,
        reason: Option<String>,
    ) {
        let payload = PolicyDecisionPayload {
            decision,
            enforcement_point: "connector_guard".to_string(),
            action: "delete_connector_instance".to_string(),
            agreement_id: None,
            transfer_id: None,
            dataset_id: Some(connector_id.to_string()),
            policy_id: Some("active_transfers_guard".to_string()),
            policy_digest: None,
            peer_id: None,
            obligations_triggered: Vec::new(),
            reason,
        };
        if let Err(err) = TypedEventSink::emit(&*self.events, &ctx, &payload).await {
            tracing::warn!(error = %err, "Failed to emit connector guard policy decision");
        }
    }

    pub fn with_transfer_guard(mut self, guard: Arc<dyn ActiveTransferGuard>) -> Self {
        self.transfer_guard = Some(guard);
        self
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
            user_id: model.user_id.clone(),
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

use ymir::oauth::{OwnedTrait, Owner, OwnerScope};
use ymir::types::oauth::UserInfo;

#[async_trait::async_trait]
impl ConnectorInstanceServiceTrait for ConnectorInstanceService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_instance_by_id(
        &self,
        user: &UserInfo,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        let id_str = id.to_string();
        if let Some(cached) = self.read_by_id(user, &id_str) {
            return Ok(Some(cached));
        }
        let instance = self
            .repo
            .get_instances_repo()
            .get_instance_by_id(&OwnerScope::seeing(user), &id_str)
            .await?;

        let dto = instance.map(Self::map_model_to_dto).transpose()?;
        if let Some(d) = &dto {
            self.write_l1(user, None, d);
        }
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_instance_by_distribution(
        &self,
        user: &UserInfo,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        let dist_id_str = distribution_id.to_string();
        if let Some(cached) = self.read_by_distro(user, &dist_id_str) {
            return Ok(Some(cached));
        }

        let relation = self
            .repo
            .get_distro_relation_repo()
            .get_relation_by_distribution(&OwnerScope::seeing(user), &dist_id_str)
            .await?;

        let relation = match relation {
            Some(r) => r,
            None => return Ok(None),
        };

        let result = self
            .repo
            .get_instances_repo()
            .get_instance_by_id(
                &OwnerScope::seeing(user),
                &relation.connector_instance_id,
            )
            .await?;

        let dto = result.map(Self::map_model_to_dto).transpose()?;
        if let Some(d) = &dto {
            self.write_l1(user, Some(&dist_id_str), d);
        }
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn upsert_instance(
        &self,
        user: &UserInfo,
        instance_dto: &mut ConnectorInstantiationDto,
    ) -> Outcome<ConnectorInstanceDto> {
        let owner = Owner::of(user, instance_dto.visibility.clone());

        // Any template the user sees, the public ones included.
        let template_model = self
            .repo
            .get_templates_repo()
            .get_template_by_name_and_version(
                &OwnerScope::seeing(user),
                &instance_dto.template_name,
                &instance_dto.template_version,
            )
            .await?
            .ok_or_else(|| {
                Errors::missing_resource(
                    format!(
                        "{}/{}",
                        instance_dto.template_name, instance_dto.template_version
                    ),
                    "connector template not found",
                    None,
                )
            })?;
        let mut connector_template_dto: ConnectorTemplateDto =
            serde_json::from_value(template_model.spec.clone())?;
        connector_template_dto.metadata.name = Some(template_model.name.clone());
        connector_template_dto.metadata.version = Some(template_model.version.clone());
        connector_template_dto.metadata.author = Some(template_model.author.clone());
        if connector_template_dto.metadata.description.is_none() {
            connector_template_dto.metadata.description = template_model
                .spec
                .get("description")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
        }

        // fetch distribution or error
        let distribution_id = instance_dto.distribution_id.to_string();
        self.distribution_facade
            .get_distribution(user, &instance_dto.distribution_id)
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
        let mut connector_resolved =
            InstanceParametersResolver::new(&connector_template_dto, &params).resolve()?;
        connector_resolved.validate_resolved()?;
        if let Some(instance_meta) = &instance_dto.metadata {
            if instance_meta.description.is_some() {
                connector_resolved.metadata.description = instance_meta.description.clone();
            }
        }

        // prepare data
        let metadata_json = connector_resolved.metadata.clone();
        let params_json = connector_resolved.parameters.clone();
        let authentication = connector_resolved.authentication.clone();
        let interaction = connector_resolved.interaction.clone();

        // dry run
        if instance_dto.dry_run {
            return Ok(ConnectorInstanceDto {
                id: Urn::from_str("urn:conector-instance:dry-run")?,
                user_id: owner.user_id.clone(),
                metadata: metadata_json,
                authentication_config: authentication,
                interaction,
                distribution_id: instance_dto.distribution_id.clone(),
            });
        }

        // persist instance
        let new_instance = connector_instances::NewConnectorInstanceModel {
            id: None,
            owner: owner.clone(),
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
            .get_relation_by_distribution(&OwnerScope::acting(user), &distribution_id)
            .await?;
        let action = match instance_distro_relation {
            None => {
                self.repo
                    .get_distro_relation_repo()
                    .create_relation(&owner, &distribution_id, &saved_model.id)
                    .await?;
                ResourceAction::Created
            }
            Some(_) => {
                self.repo
                    .get_distro_relation_repo()
                    .update_relation(&OwnerScope::acting(user), &distribution_id, &saved_model.id)
                    .await?;
                ResourceAction::Updated
            }
        };

        let owner = saved_model.owner();
        let result = Self::map_model_to_dto(saved_model)?;
        self.evict_instance(&result.id.to_string(), Some(&distribution_id));
        let connector_type = match &result.interaction {
            InteractionConfig::Pull(_) => "pull",
            InteractionConfig::Push(_) => "push",
        };
        let instance_name = result
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| result.id.to_string());
        self.emit_lifecycle(
            EventContext::for_owner(owner).with_actor(EventActor::human(user)),
            action,
            result.id.to_string(),
            instance_name,
            Some(connector_type.to_string()),
        )
        .await;
        Ok(result)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_instance_by_id(&self, user: &UserInfo, id: &Urn) -> Outcome<()> {
        let guard_checked = if let Some(guard) = &self.transfer_guard {
            let active = guard.count_active_transfers(id).await?;
            if active > 0 {
                let reason = format!(
                    "connector instance {id} is referenced by {active} active transfer(s)"
                );
                self.emit_guard_decision(
                    EventContext::for_user(user).with_correlation(id.to_string()),
                    PolicyDecisionOutcome::Deny,
                    &id.to_string(),
                    Some(reason),
                )
                .await;
                return Err(ConnectorErrors::conflict(format!(
                    "connector instance {id} is referenced by {active} active transfer(s); terminate them before deleting the instance"
                )));
            }
            true
        } else {
            false
        };

        let id_str = id.to_string();

        let _ = self
            .repo
            .get_distro_relation_repo()
            .delete_relation_by_instance(&OwnerScope::acting(user), &id_str)
            .await;

        let owner = self
            .repo
            .get_instances_repo()
            .delete_instance_by_id(&OwnerScope::acting(user), &id_str)
            .await?;
        self.evict_instance(&id_str, None);
        if guard_checked {
            self.emit_guard_decision(
                EventContext::for_owner(owner.clone())
                    .with_actor(EventActor::human(user))
                    .with_correlation(id_str.clone()),
                PolicyDecisionOutcome::Permit,
                &id_str,
                None,
            )
            .await;
        }
        self.emit_lifecycle(
            EventContext::for_owner(owner).with_actor(EventActor::human(user)),
            ResourceAction::Deleted,
            id_str.clone(),
            id_str,
            None,
        )
        .await;
        Ok(())
    }
}
