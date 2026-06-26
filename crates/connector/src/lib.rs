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
pub(crate) mod entities;
pub(crate) mod facades;
pub(crate) mod grpc;
pub(crate) mod http;
pub(crate) mod setup;

pub use data::entities::connector_instances::Model as ConnectorInstanceModel;
pub use data::migrations::get_connector_migrations;
pub use data::repo_traits::connector_instance_repo::ConnectorInstanceRepoTrait;
pub use entities::connector_instance;
pub use entities::connector_instance::{
    ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto,
};
pub use entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
pub use entities::parameters::TemplateVecString;
pub use entities::resource::{HttpSpec, ProtocolSpec};
pub use setup::ConnectorSetup;

pub use entities::auth_config::{
    ApiKeyLocation, AuthenticationConfig, BasicAuthConfig, OAuthGrantType, TokenExpireAction,
};
pub use entities::common::secret_management::{SecretSource, SecretString};
pub use entities::parameters::runtime_parameters_resolver::RuntimeParametersResolver;

#[cfg(test)]
pub use entities::connector_instance::MockConnectorInstanceTrait;
pub use entities::connector_template::ConnectorMetadata;
pub use entities::parameters::keystore_lookup::KeystoreLookup;
pub use entities::parameters::{template_runtime_parameter_regex, template_runtime_secret_regex};
