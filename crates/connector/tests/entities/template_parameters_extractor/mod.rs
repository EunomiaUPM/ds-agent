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

//! TemplateParametersExtractor: which `{{__NAME__}}` placeholders it finds, split by where they
//! appear. `run` walks a whole template and returns the names found.

mod auth;
mod fields;
mod http;
mod kafka;

use connector::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use connector::entities::parameters::{FoundParameterType, TemplateMapString};
use connector::TemplateVecString;
use std::collections::HashMap;

use connector::entities::auth_config::{ApiKeyLocation, BasicAuthConfig, OAuthGrantType};
use connector::entities::common::secret_management::{SecretSource, SecretString};
use connector::entities::connector_template::ConnectorTemplateDto;
use connector::entities::parameters::template_parameters_extractor::TemplateParametersExtractor;
use connector::entities::resource::KafkaSpec;
use connector::{
    AuthenticationConfig, ConnectorMetadata, HttpSpec, InteractionConfig, ProtocolSpec,
    PullLifecycle, PushLifecycle,
};

fn run(mut dto: ConnectorTemplateDto) -> Vec<String> {
    let mut extractor = TemplateParametersExtractor::new();
    extractor.walk(&mut dto).expect("template walks");
    extractor
        .found_parameters()
        .iter()
        .map(|fp| fp.name.clone())
        .collect()
}
