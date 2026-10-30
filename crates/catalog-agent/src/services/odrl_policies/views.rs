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

//! An offer as the management API returns it.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use common::dsp_common::odrl::OdrlPolicyInfo;
use common::eunomia::document::JsonLd;

use crate::entities::odrl_offer::OdrlOffer;
use crate::entities::policy_templates::ControlPoint;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OdrlPolicyView {
    pub id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub odrl_offer: Value,
    pub entity: String,
    pub entity_type: String,
    pub created_at: DateTime<Utc>,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub instantiation_parameters: Option<Value>,
    pub description: Option<String>,
    /// The parts a composed offer was made from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composition: Option<Value>,
    /// Where its pieces are meant to apply; informative, nothing evaluates it.
    pub control_points: Vec<ControlPoint>,
}

impl OdrlPolicyView {
    pub fn assemble(offer: OdrlOffer) -> Self {
        Self {
            id: offer.id.to_string(),
            user_id: offer.owner.user_id,
            user_role: offer.owner.role,
            visibility: offer.owner.visibility,
            odrl_offer: JsonLd::<OdrlPolicyInfo>::tag(offer.odrl_offer),
            entity: offer.entity.to_string(),
            entity_type: offer.entity_type,
            created_at: offer.created_at,
            source_template_id: offer.source_template_id,
            source_template_version: offer.source_template_version,
            instantiation_parameters: offer.instantiation_parameters,
            description: offer.description,
            control_points: offer
                .composition
                .as_ref()
                .and_then(|c| c.get("controlPoints"))
                .and_then(|points| serde_json::from_value(points.clone()).ok())
                .unwrap_or_default(),
            composition: offer.composition.and_then(|mut c| c.get_mut("parts").map(serde_json::Value::take)),
        }
    }
}
