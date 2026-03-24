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

//! Injects declared default values for parameters the user left unset.
//!
//! This is phase 1b of the parameter enrichment pipeline — it runs after
//! [`SysParameterEnricher`] and before [`TemplateParametersResolver`].
//!
//! [`SysParameterEnricher`]: super::sys_parameter_enricher::SysParameterEnricher
//! [`TemplateParametersResolver`]: super::template_parameters_resolver::TemplateParametersResolver
mod test;

use crate::entities::parameters::parameters::ParameterDefinition;
use crate::entities::parameters::ParameterEnricher;
use serde_json::Value;
use std::collections::HashMap;
use ymir::errors::Outcome;
use crate::entities::parameters::default_parameters_injector::DefaultParametersInjector;

/// Enriches the parameter map with declared default values.
///
/// For each [`ParameterDefinition`] that carries a `default_value`, the
/// enricher inserts that value only when the key is **absent** from the
/// parameter map — it never overwrites a value supplied by the user or by an
/// earlier pipeline step (e.g. [`SysParameterEnricher`]).
///
/// [`SysParameterEnricher`]: super::sys_parameter_enricher::SysParameterEnricher
///
/// # Example
///
/// ```ignore
/// SysParameterEnricher::new(&template_spec, &own_url).enrich(&mut params)?;
/// DefaultParameterEnricher::new(&template_spec.parameters).enrich(&mut params)?;
/// ```
pub struct DefaultParameterEnricher<'a> {
    definitions: &'a [ParameterDefinition],
}

impl<'a> DefaultParameterEnricher<'a> {
    pub fn new(definitions: &'a [ParameterDefinition]) -> Self {
        Self { definitions }
    }
}

impl ParameterEnricher for DefaultParameterEnricher<'_> {
    /// Fills in the declared `default_value` for every parameter that has one
    /// and whose key is not yet present in `params`.
    fn enrich(&self, params: &mut HashMap<String, Value>) -> Outcome<()> {
        DefaultParametersInjector::inject(self.definitions, params)
    }
}
