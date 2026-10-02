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

use super::{FoundParameter, ParameterDefinition, ParameterType, SysParameterType};
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::template_parameters_extractor::TemplateParametersExtractor;
use crate::ConnectorInstanceDto;
use serde_json::json;
use std::collections::HashMap;
use ymir::errors::{Errors, Outcome};

pub struct InstanceParametersMap {
    inner: HashMap<String, serde_json::Value>,
}

impl InstanceParametersMap {
    pub fn collect(&self) -> HashMap<String, serde_json::Value> {
        self.inner.clone()
    }
}

pub struct InstanceParametersMapBuilder {
    own_url: String,
    params: HashMap<String, serde_json::Value>,
}

impl InstanceParametersMapBuilder {
    pub fn new(own_url: String) -> Self {
        Self {
            own_url,
            params: HashMap::new(),
        }
    }

    pub fn with_instance_parameters(
        mut self,
        instance_parameters: &HashMap<String, serde_json::Value>,
    ) -> Self {
        for (k, v) in instance_parameters {
            self.params.entry(k.clone()).or_insert_with(|| v.clone());
        }
        self
    }
    pub fn with_default_parameters(mut self, parameters: &[ParameterDefinition]) -> Outcome<Self> {
        for def in parameters {
            if self.params.contains_key(&def.name) {
                continue;
            }
            if let Some(default_str) = &def.default_value {
                let json_value = Self::cast_default_value(&def.name, &def.param_type, default_str)?;
                self.params.insert(def.name.clone(), json_value);
            }
        }
        Ok(self)
    }

    pub fn with_system_parameters(
        mut self,
        connector_template: &mut ConnectorTemplateDto,
    ) -> Outcome<Self> {
        let mut extractor = TemplateParametersExtractor::new().just_system_parameters();
        extractor.walk(connector_template)?;
        let parameters_found = extractor.found_parameters();
        for found in parameters_found {
            let value = self.resolve_sys_value(found)?;
            self.params.entry(found.name.clone()).or_insert(value);
        }
        Ok(self)
    }

    pub fn with_runtime_parameters(
        self,
        _connector_instance: &ConnectorInstanceDto,
    ) -> Outcome<Self> {
        Ok(self)
    }

    pub fn build(self) -> InstanceParametersMap {
        InstanceParametersMap { inner: self.params }
    }

    fn cast_default_value(
        param_name: &str,
        param_type: &ParameterType,
        raw_val: &str,
    ) -> Outcome<serde_json::Value> {
        match param_type {
            ParameterType::String => Ok(json!(raw_val)),
            ParameterType::Int => {
                let parsed = raw_val.parse::<i64>().map_err(|_| {
                    Errors::crazy(
                        format!(
                            "Template Definition Error: Default value '{raw_val}' for param '{param_name}' is not a valid Integer."
                        ),
                        None,
                    )
                })?;
                Ok(json!(parsed))
            }
            ParameterType::Boolean => {
                let parsed = raw_val.to_lowercase().parse::<bool>().map_err(|_| {
                    Errors::crazy(
                        format!(
                            "Template Definition Error: Default value '{raw_val}' for param '{param_name}' is not a valid Boolean."
                        ),
                        None,
                    )
                })?;
                Ok(json!(parsed))
            }
            ParameterType::VecString | ParameterType::MapStringString => {
                let parsed: serde_json::Value = serde_json::from_str(raw_val).map_err(|e| {
                    Errors::crazy(format!("Template Definition Error: Default value '{raw_val}' for complex param '{param_name}' is not valid JSON: {e}"), None)
                })?;

                if matches!(param_type, ParameterType::VecString) && !parsed.is_array() {
                    return Err(Errors::crazy(
                        format!("Default value for '{param_name}' must be a JSON Array"),
                        None,
                    ));
                }
                if matches!(param_type, ParameterType::MapStringString) && !parsed.is_object() {
                    return Err(Errors::crazy(
                        format!("Default value for '{param_name}' must be a JSON Object"),
                        None,
                    ));
                }

                Ok(parsed)
            }
        }
    }

    fn resolve_sys_value(&self, found_parameter: &FoundParameter) -> Outcome<serde_json::Value> {
        let sys_parameter_parsed = found_parameter.name.parse::<SysParameterType>()?;
        match sys_parameter_parsed {
            SysParameterType::SysUrn => {
                let nss = uuid::Uuid::new_v4().to_string();
                let urn = urn::UrnBuilder::new("uuid", &nss).build()?;
                Ok(json!(urn.to_string()))
            }
            SysParameterType::SysToken => Ok(json!(uuid::Uuid::new_v4().to_string())),
            SysParameterType::SysTimestamp => Ok(json!(chrono::Utc::now().timestamp())),
            SysParameterType::SysIso8601 => Ok(json!(chrono::Utc::now().to_rfc3339())),
            SysParameterType::SysOwnUrl {
                host_docker_internal: false,
            } => Ok(json!(self.own_url)),
            SysParameterType::SysOwnUrl {
                host_docker_internal: true,
            } => {
                let docker_url = self
                    .own_url
                    .replace("localhost", "host.docker.internal")
                    .replace("127.0.0.1", "host.docker.internal");
                Ok(json!(docker_url))
            }
        }
    }
}
