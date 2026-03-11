use crate::data::repo_traits::connector_distro_relation_repo::ConnectorDistroRelationRepoTrait;
use crate::data::repo_traits::connector_instance_repo::ConnectorInstanceRepoTrait;
use crate::data::repo_traits::connector_template_repo::ConnectorTemplateRepoTrait;
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait ConnectorRepoTrait: Send + Sync {
    fn get_templates_repo(&self) -> Arc<dyn ConnectorTemplateRepoTrait>;
    fn get_instances_repo(&self) -> Arc<dyn ConnectorInstanceRepoTrait>;
    fn get_distro_relation_repo(&self) -> Arc<dyn ConnectorDistroRelationRepoTrait>;
}
