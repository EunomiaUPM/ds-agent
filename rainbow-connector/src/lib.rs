pub(crate) mod data;
pub(crate) mod entities;
pub(crate) mod facades;
pub(crate) mod grpc;
pub(crate) mod http;
pub(crate) mod setup;

pub use data::migrations::get_connector_migrations;
pub use setup::ConnectorSetup;
pub use data::repo_traits::connector_instance_repo::ConnectorInstanceRepoTrait;
pub use data::entities::connector_instances::Model as ConnectorInstanceModel;
pub use entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
pub use entities::resource::{ProtocolSpec, HttpSpec};
pub use entities::common::parameters::TemplateVecString;
pub use entities::connector_instance::{ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto};
pub use entities::connector_instance;
pub use entities::connector_instance::resolver::TemplateResolver;
pub use entities::common::parameters::TemplateMutable;

#[cfg(test)]
pub use entities::connector_instance::MockConnectorInstanceTrait;
pub use entities::connector_template::ConnectorMetadata;
pub use entities::auth_config::AuthenticationConfig;
