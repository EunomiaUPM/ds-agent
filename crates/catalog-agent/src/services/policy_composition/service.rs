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

//! Composition service over the template and offer services.

use std::collections::HashMap;
use std::sync::Arc;

use common::dsp_common::odrl::OdrlPolicyInfo;
use common::eunomia::document::JsonLd;
use serde_json::{json, Value};
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::entities::commands::NewOdrlOfferCommand;
use crate::entities::odrl_offer::{CatalogEntityTypes, OdrlOffer};
use crate::entities::policy_composition::composer::OfferComposer;
use crate::entities::policy_composition::{NewPolicyCompositionDto, OfferPart};
use crate::entities::policy_instantiation::NewPolicyInstantiationDto;
use crate::entities::policy_templates::ControlPoint;
use crate::services::policy_composition::PolicyCompositionServiceTrait;
use crate::services::policy_templates::views::PolicyPreviewView;
use crate::setup::context::AppContext;

/// A composition worked out: its ODRL, where it applies and the template versions it used.
struct Composed {
    odrl: OdrlPolicyInfo,
    control_points: Vec<ControlPoint>,
    references: Vec<(String, String)>,
}

/// Where a composed offer goes, and what it remembers of how it was made.
struct Target<'r> {
    entity_id: &'r Urn,
    entity_type: CatalogEntityTypes,
    description: Option<String>,
    visibility: Option<ymir::types::participants::Visibility>,
    parts: &'r [Value],
    source_template: Option<(String, String, Value)>,
}

pub struct PolicyCompositionService {
    ctx: Arc<AppContext>,
}

impl PolicyCompositionService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    async fn build(&self, user: &UserInfo, parts: &[Value]) -> Outcome<Composed> {
        if parts.is_empty() {
            return Err(Errors::validation("a composition needs at least one part", None));
        }
        let parsed = parts
            .iter()
            .enumerate()
            .map(|(i, part)| OfferPart::parse(part, &format!("parts[{i}]")))
            .collect::<Outcome<Vec<_>>>()?;
        let library = self.ctx.templates().library(user, OfferComposer::keys(&parsed)?).await?;
        let offer_ids: Vec<Urn> = parsed
            .iter()
            .filter_map(|part| match part {
                OfferPart::Offer(id) => Some(id.clone()),
                _ => None,
            })
            .collect();
        let offers: HashMap<Urn, Value> = if offer_ids.is_empty() {
            HashMap::new()
        } else {
            self.ctx
                .offers()
                .batch(user, &offer_ids)
                .await?
                .into_iter()
                .map(|offer| (offer.id, offer.odrl_offer))
                .collect()
        };
        let composer = OfferComposer::new(&library, &offers);
        let flat = composer.compose(&parsed)?;
        let odrl: OdrlPolicyInfo = serde_json::from_value(flat)
            .map_err(|e| Errors::validation(format!("the composition is not valid ODRL: {e}"), None))?;
        Ok(Composed {
            odrl,
            control_points: composer.control_points(&parsed)?.into_iter().collect(),
            references: library.keys(),
        })
    }

    async fn store(&self, user: &UserInfo, target: Target<'_>) -> Outcome<OdrlOffer> {
        let composed = self.build(user, target.parts).await?;
        let (source_template_id, source_template_version, instantiation_parameters) = match target.source_template {
            Some((id, version, values)) => (Some(id), Some(version), Some(values)),
            None => (None, None, None),
        };
        self.ctx
            .offers()
            .create(
                user,
                &NewOdrlOfferCommand {
                    id: None,
                    visibility: target.visibility,
                    owner: None,
                    odrl_offer: JsonLd(composed.odrl),
                    entity_id: target.entity_id.clone(),
                    entity_type: target.entity_type,
                    source_template_id,
                    source_template_version,
                    instantiation_parameters,
                    description: target.description,
                    composition: Some(json!({
                        "parts": target.parts,
                        "controlPoints": composed.control_points,
                    })),
                    references: composed.references,
                },
            )
            .await
    }
}

#[async_trait::async_trait]
impl PolicyCompositionServiceTrait for PolicyCompositionService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn compose(&self, user: &UserInfo, request: &NewPolicyCompositionDto) -> Outcome<OdrlOffer> {
        self.store(
            user,
            Target {
                entity_id: &request.entity_id,
                entity_type: request.entity_type,
                description: request.description.clone(),
                visibility: request.visibility.clone(),
                parts: &request.parts,
                source_template: None,
            },
        )
        .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn preview(&self, user: &UserInfo, parts: &[Value]) -> Outcome<PolicyPreviewView> {
        let composed = self.build(user, parts).await?;
        Ok(PolicyPreviewView {
            odrl: serde_json::to_value(&composed.odrl)?,
            control_points: composed.control_points,
        })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn instantiate(&self, user: &UserInfo, request: &NewPolicyInstantiationDto) -> Outcome<OdrlOffer> {
        let values: serde_json::Map<String, Value> =
            request.parameters.iter().map(|(k, v)| (k.clone(), v.to_json())).collect();
        let parts = [json!({
            "@ref": format!("{}@{}", request.id, request.version),
            "with": values.clone(),
        })];
        self.store(
            user,
            Target {
                entity_id: &request.entity_id,
                entity_type: request.entity_type,
                description: request.description.clone(),
                visibility: None,
                parts: &parts,
                source_template: Some((request.id.clone(), request.version.clone(), Value::Object(values))),
            },
        )
        .await
    }
}
