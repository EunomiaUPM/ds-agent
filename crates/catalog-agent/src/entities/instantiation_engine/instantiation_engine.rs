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

use crate::entities::instantiation_engine::{NewPolicyInstantiationDto, PolicyInstantiationTrait};
use crate::entities::odrl_policies::{NewOdrlPolicyDto, OdrlPolicyEntityTrait};
use crate::entities::policy_templates::{PolicyTemplateDto, PolicyTemplateEntityTrait};
use crate::OdrlPolicyDto;
use common::dsp_common::odrl::OdrlPolicyInfo;
use common::errors::{CommonErrors, ErrorLog};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;
use ymir::errors::{Errors, Outcome};

pub struct PolicyInstantiationEngine {
    odrl_policy_service: Arc<dyn OdrlPolicyEntityTrait>,
    policy_templates_service: Arc<dyn PolicyTemplateEntityTrait>,
}

impl PolicyInstantiationEngine {
    pub fn new(
        odrl_policy_service: Arc<dyn OdrlPolicyEntityTrait>,
        policy_templates_service: Arc<dyn PolicyTemplateEntityTrait>,
    ) -> PolicyInstantiationEngine {
        Self { odrl_policy_service, policy_templates_service }
    }

    fn substitute_variables_recursive(
        value: &mut Value,
        params: &HashMap<String, Value>,
    ) -> Outcome<()> {
        match value {
            Value::String(s) => {
                if s.starts_with('$') {
                    if let Some(replacement) = params.get(s) {
                        *value = replacement.clone();
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::substitute_variables_recursive(item, params)?;
                }
            }
            Value::Object(map) => {
                for (_, v) in map {
                    Self::substitute_variables_recursive(v, params)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl PolicyInstantiationTrait for PolicyInstantiationEngine {
    async fn instantiate_policy(
        &self,
        instantiation_request: &NewPolicyInstantiationDto,
    ) -> Outcome<OdrlPolicyDto> {
        // fetch policy template
        let policy_template = self
            .policy_templates_service
            .get_policies_template_by_version_and_id(
                &instantiation_request.id,
                &instantiation_request.version,
            )
            .await?
            .ok_or_else(|| {
                Errors::missing_resource(
                    "PolicyTemplate",
                    &format!(
                        "ID: {} Version: {}",
                        instantiation_request.id, instantiation_request.version
                    ),
                    None,
                )
            })?;

        // validate
        instantiation_request.validate_instantiation_request(&policy_template)?;

        // merge params
        let mut final_params = HashMap::with_capacity(policy_template.parameters.len());
        for (key, def) in &policy_template.parameters {
            if let Some(default_val) = &def.default_value {
                let val_json = serde_json::to_value(default_val).unwrap_or(Value::Null);
                final_params.insert(key.clone(), val_json);
            }
        }
        for (key, val) in &instantiation_request.parameters {
            let val_json = serde_json::to_value(val)?;
            final_params.insert(key.clone(), val_json);
        }

        let mut odrl_content_json = serde_json::to_value(&policy_template.content)?;
        Self::substitute_variables_recursive(&mut odrl_content_json, &final_params)?;

        // create policy info
        let final_odrl: OdrlPolicyInfo = serde_json::from_value(odrl_content_json)?;

        // create offer
        let created_offer = self
            .odrl_policy_service
            .create_odrl_offer(&NewOdrlPolicyDto {
                id: None,
                odrl_offer: final_odrl,
                entity_id: instantiation_request.entity_id.clone(),
                entity_type: instantiation_request.entity_type.clone(),
                source_template_id: Some(instantiation_request.id.clone()),
                source_template_version: Some(instantiation_request.version.clone()),
                instantiation_parameters: Some(serde_json::to_value(
                    &instantiation_request.parameters,
                )?),
                description: instantiation_request.description.clone(),
            })
            .await?;

        Ok(created_offer)
    }
}
