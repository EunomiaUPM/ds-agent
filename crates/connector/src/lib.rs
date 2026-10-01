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

pub(crate) mod data;
pub mod entities;
pub mod facades;
pub(crate) mod http;
pub mod services;
pub(crate) mod setup;

pub const EVENT_DOMAIN: &str = "connector";
pub const EVENT_PREFIX: &str = "connector:";

pub use data::entities::connector_instances::Model as ConnectorInstanceModel;
pub use data::migrations::get_connector_migrations;
pub use data::repo_traits::connector_instance_repo::ConnectorInstanceRepoTrait;
pub use entities::connector_instance;
pub use entities::connector_instance::{ConnectorInstanceDto, ConnectorInstantiationDto};
pub use entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
pub use entities::parameters::TemplateVecString;
pub use entities::resource::{HttpSpec, ProtocolSpec};
pub use facades::catalog_facade::CatalogFacadeTrait;
pub use facades::connector_instance_facade::local::ConnectorInstanceLocalFacade;
pub use facades::connector_instance_facade::remote::ConnectorInstanceRemoteFacade;
pub use facades::connector_instance_facade::{
    ConnectorInstanceFacadeTrait, MockConnectorInstanceFacadeTrait,
};
pub use setup::{ConnectorModule, ConnectorPorts};

pub use entities::auth_config::{
    ApiKeyLocation, AuthenticationConfig, BasicAuthConfig, OAuthGrantType, TokenExpireAction,
};
pub use entities::common::secret_management::{SecretSource, SecretString};
pub use entities::parameters::runtime_parameters_resolver::RuntimeParametersResolver;

pub use entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
pub use entities::parameters::keystore_lookup::KeystoreLookup;
pub use entities::parameters::{template_runtime_parameter_regex, template_runtime_secret_regex};
pub use services::connector_instance::ConnectorInstanceServiceTrait;
pub use services::connector_template::ConnectorTemplateServiceTrait;
#[cfg(test)]
pub use services::connector_template::MockConnectorTemplateServiceTrait;
