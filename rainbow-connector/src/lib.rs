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
pub use entities::parameters::parameters::TemplateVecString;
pub use entities::parameters::template_parameters_resolver::TemplateParametersResolver;
pub use entities::parameters::template_resolver_visitor::TemplateResolverVisitor;
pub use entities::resource::{HttpSpec, ProtocolSpec};
pub use setup::ConnectorSetup;

pub use entities::auth_config::AuthenticationConfig;
#[cfg(test)]
pub use entities::connector_instance::MockConnectorInstanceTrait;
pub use entities::connector_template::ConnectorMetadata;
