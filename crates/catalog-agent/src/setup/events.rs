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

//! Canonical event adapters translating catalog domain writes into typed `ymir-events` envelopes.

use events::schemas::dataspace::{CatalogResourcePayload, PolicyDefinitionPayload, ResourceAction};
use events::{
    EventActor, EventContext, EventIntegrity, EventPayload, EventSinkTrait, TypedEventSink,
};
use serde_json::Value;
use tracing::warn;
use ymir::oauth::{OwnedTrait, Owner};
use ymir::types::oauth::UserInfo;

use crate::entities::odrl_offer::OdrlOffer;
use crate::entities::policy_templates::types::LocalizedText;
use crate::entities::policy_templates::PolicyTemplateDto;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::schemas::DataSchema;

/// Stateless adapter emitting `catalog-agent` events with compact metadata and digests.
pub struct CatalogEvents;

impl CatalogEvents {
    fn resource_parts(
        resource: &CatalogResource,
        action: ResourceAction,
        actor: Option<EventActor>,
    ) -> (EventContext, CatalogResourcePayload) {
        let payload = CatalogResourcePayload {
            action,
            resource_kind: resource.kind().as_str().to_string(),
            resource_id: resource.id().to_string(),
            title: resource.title(),
            version: Some(resource.version().to_string()),
            format: None,
            source_url: None,
        };
        let mut ctx = EventContext::for_owner(resource.owner().clone());
        if resource.kind() == ResourceKind::Dataset {
            ctx = ctx.with_dataset_id(resource.id().to_string());
        } else if let Some(dataset_id) = resource.dataset_id() {
            ctx = ctx.with_dataset_id(dataset_id.to_string());
        }
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        (ctx, payload)
    }

    /// Emits a typed lifecycle event for a catalog, dataset, distribution, or data service.
    pub async fn resource(
        sink: &dyn EventSinkTrait,
        resource: &CatalogResource,
        action: ResourceAction,
        actor: Option<EventActor>,
    ) {
        let (ctx, payload) = Self::resource_parts(resource, action, actor);
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits typed lifecycle events for multiple resources in a single batch sink call.
    pub async fn resources_batch(
        sink: &dyn EventSinkTrait,
        items: &[(&CatalogResource, ResourceAction)],
        actor: Option<&EventActor>,
    ) {
        let built: Vec<(EventContext, CatalogResourcePayload)> = items
            .iter()
            .map(|(resource, action)| Self::resource_parts(resource, action.clone(), actor.cloned()))
            .collect();
        let refs: Vec<(&EventContext, &CatalogResourcePayload)> =
            built.iter().map(|(ctx, payload)| (ctx, payload)).collect();
        if let Some((_, first)) = refs.first() {
            if let Err(err) = sink.emit_batch(&refs).await {
                warn!(
                    topic = %first.topic(),
                    count = refs.len(),
                    error = %err,
                    "catalog batch event emit failed"
                );
            }
        }
    }

    /// Maps legacy resource action labels to canonical `ResourceAction` variants.
    pub fn map_resource_action(action: &str) -> ResourceAction {
        match action {
            "create" => ResourceAction::Created,
            "delete" => ResourceAction::Deleted,
            "import" => ResourceAction::Imported,
            _ => ResourceAction::Updated,
        }
    }

    /// Emits a typed creation event for a stored JSON schema version.
    pub async fn schema_created(sink: &dyn EventSinkTrait, schema: &DataSchema) {
        let payload = CatalogResourcePayload {
            action: ResourceAction::Created,
            resource_kind: "schema".to_string(),
            resource_id: format!("{}/{}", schema.id, schema.version),
            title: Some(schema.title.clone()),
            version: Some(schema.version.to_string()),
            format: Some("json-schema".to_string()),
            source_url: None,
        };
        let ctx = EventContext::for_owner(schema.owner());
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed deletion event for a schema or schema version.
    pub async fn schema_deleted(
        sink: &dyn EventSinkTrait,
        owner: &Owner,
        deleted_id: String,
        version: Option<i32>,
        actor: EventActor,
    ) {
        let payload = CatalogResourcePayload {
            action: ResourceAction::Deleted,
            resource_kind: "schema".to_string(),
            resource_id: deleted_id,
            title: None,
            version: version.map(|v| v.to_string()),
            format: Some("json-schema".to_string()),
            source_url: None,
        };
        let ctx = EventContext::for_owner(owner.clone()).with_actor(actor);
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed policy creation event for a policy template version without full JSON dumps.
    pub async fn policy_template_created(sink: &dyn EventSinkTrait, template: &PolicyTemplateDto) {
        let (permission_count, prohibition_count, obligation_count) =
            Self::rule_counts(&template.content);
        let payload = PolicyDefinitionPayload {
            action: ResourceAction::Created,
            policy_id: format!("{}@{}", template.id, template.version),
            title: template.title.as_ref().map(Self::localized_title),
            permission_count,
            prohibition_count,
            obligation_count,
            policy_digest: Some(EventIntegrity::for_payload(&template.content).payload_digest),
        };
        let ctx = EventContext::for_owner(template.owner());
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed policy deletion event for a policy template version.
    pub async fn policy_template_deleted(
        sink: &dyn EventSinkTrait,
        user: &UserInfo,
        template_id: &str,
        version_id: &str,
    ) {
        let payload = PolicyDefinitionPayload {
            action: ResourceAction::Deleted,
            policy_id: format!("{template_id}@{version_id}"),
            title: None,
            permission_count: 0,
            prohibition_count: 0,
            obligation_count: 0,
            policy_digest: None,
        };
        let ctx = EventContext::for_owner(Owner::private(user)).with_actor(EventActor::human(user));
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed lifecycle event for an ODRL offer using canonical digest and rule counts.
    pub async fn odrl_offer(
        sink: &dyn EventSinkTrait,
        offer: &OdrlOffer,
        action: ResourceAction,
        actor: Option<EventActor>,
    ) {
        let (permission_count, prohibition_count, obligation_count) =
            Self::rule_counts(&offer.odrl_offer);
        let payload = PolicyDefinitionPayload {
            action,
            policy_id: offer.id.to_string(),
            title: offer.description.clone(),
            permission_count,
            prohibition_count,
            obligation_count,
            policy_digest: Some(EventIntegrity::for_payload(&offer.odrl_offer).payload_digest),
        };
        let mut ctx = EventContext::for_owner(offer.owner());
        if offer.entity_type == "Dataset" {
            ctx = ctx.with_dataset_id(offer.entity.to_string());
        }
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    fn rule_counts(policy: &Value) -> (usize, usize, usize) {
        let count_key = |keys: &[&str]| -> usize {
            keys.iter()
                .filter_map(|k| policy.get(*k))
                .map(|v| v.as_array().map_or(1, Vec::len))
                .sum()
        };
        (
            count_key(&["permission", "odrl:permission"]),
            count_key(&["prohibition", "odrl:prohibition"]),
            count_key(&["obligation", "odrl:obligation"]),
        )
    }

    fn localized_title(text: &LocalizedText) -> String {
        match text {
            LocalizedText::Single(s) => s.clone(),
            LocalizedText::Multiple(items) => {
                items.first().map(|item| item.value.clone()).unwrap_or_default()
            }
        }
    }

    async fn emit_typed<P: EventPayload>(sink: &dyn EventSinkTrait, ctx: &EventContext, payload: &P) {
        if let Err(err) = sink.emit(ctx, payload).await {
            warn!(topic = %payload.topic(), error = %err, "catalog event emit failed");
        }
    }
}
