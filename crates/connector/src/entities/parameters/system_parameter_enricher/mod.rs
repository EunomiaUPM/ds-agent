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

//! Injects `SYS_*` runtime values for every `{{__SYS_*__}}` placeholder that
//! is actually referenced inside a [`ConnectorTemplateDto`].
//!
//! # How it works
//!
//! The enricher performs two passes over the template:
//!
//! 1. **Discovery** — [`ParameterExtractorVisitor`] walks the whole DTO and
//!    feeds each templatable field to [`SystemParameterExtractor`], which
//!    collects every `{{__SYS_*__}}` match together with its
//!    [`SysParameterType`].
//!
//! 2. **Injection** — for each discovered placeholder, a runtime value is
//!    computed and inserted into the parameter map via [`HashMap::entry`] so
//!    that user-supplied overrides are never silently replaced.
//!
//! # `SysOwnUrl` resolution
//!
//! `{{__SYS_OWN_URL__}}` resolves to the service's own base URL as configured
//! (e.g. `https://my-connector.example.com:8080`).
//!
//! `{{__SYS_OWN_URL_DOCKER__}}` resolves to the same URL but with `localhost`
//! and `127.0.0.1` replaced by `host.docker.internal`, making the address
//! reachable from inside a Docker container.
//!
//! Both require the `own_url` string to be injected at construction time via
//! [`SysParameterEnricher::new`].
//!
//! [`ConnectorTemplateDto`]: crate::entities::connector_template::ConnectorTemplateDto
mod test;

use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::parameters::SysParameterType;
use crate::entities::parameters::system_parameter_extractor::SystemParameterExtractor;
use crate::entities::parameters::template_parameters_visitor::ParameterExtractorVisitor;
use crate::entities::parameters::ParameterEnricher;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use ymir::errors::Outcome;

/// Enriches the parameter map with `SYS_*` runtime values.
///
/// Construct with a reference to the connector template and the service's own
/// base URL, then call [`ParameterEnricher::enrich`].
///
/// # Example
///
/// ```ignore
/// let mut params = instance_dto.parameters.clone();
/// SysParameterEnricher::new(&template_spec, &self.own_url).enrich(&mut params)?;
/// ```
pub struct SysParameterEnricher<'a> {
    template: &'a ConnectorTemplateDto,
    /// The service's own base URL (e.g. `https://my-connector.example.com:8080`).
    /// Used to resolve `{{__SYS_OWN_URL__}}` and `{{__SYS_OWN_URL_DOCKER__}}`.
    own_url: &'a str,
}

impl<'a> SysParameterEnricher<'a> {
    pub fn new(template: &'a ConnectorTemplateDto, own_url: &'a str) -> Self {
        Self { template, own_url }
    }

    /// Compute the runtime [`Value`] for a given [`SysParameterType`].
    fn resolve_sys_value(&self, content_type: &SysParameterType) -> Option<Value> {
        match content_type {
            SysParameterType::SysUrn => {
                let nss = uuid::Uuid::new_v4().to_string();
                let urn = urn::UrnBuilder::new("uuid", &nss).build().ok()?;
                Some(json!(urn.to_string()))
            }
            SysParameterType::SysToken => Some(json!(uuid::Uuid::new_v4().to_string())),
            SysParameterType::SysTimestamp => Some(json!(Utc::now().timestamp())),
            SysParameterType::SysIso8601 => Some(json!(Utc::now().to_rfc3339())),
            // The regular own URL — returned as-is from config.
            SysParameterType::SysOwnUrl {
                host_docker_internal: false,
            } => Some(json!(self.own_url)),
            // The Docker variant — replace localhost / 127.0.0.1 so that the
            // address is reachable from inside a container.
            SysParameterType::SysOwnUrl {
                host_docker_internal: true,
            } => {
                let docker_url = self
                    .own_url
                    .replace("localhost", "host.docker.internal")
                    .replace("127.0.0.1", "host.docker.internal");
                Some(json!(docker_url))
            }
        }
    }
}

impl ParameterEnricher for SysParameterEnricher<'_> {
    /// Scans the template for `{{__SYS_*__}}` placeholders and inserts a
    /// runtime value for each one that does not already exist in `params`.
    fn enrich(&self, params: &mut HashMap<String, Value>) -> Outcome<()> {
        let mut extractor = SystemParameterExtractor::new();
        // Clone the template so we can satisfy the &mut requirement of
        // ParameterExtractorVisitor without modifying the original.
        let mut tmpl = self.template.clone();
        ParameterExtractorVisitor::new(&mut extractor).extract(&mut tmpl);

        for found in extractor.found_sys_parameters() {
            if let Some(value) = self.resolve_sys_value(&found.content_type) {
                params.entry(found.name.clone()).or_insert(value);
            }
        }
        Ok(())
    }
}
