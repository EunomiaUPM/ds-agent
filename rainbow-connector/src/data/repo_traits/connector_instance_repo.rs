use crate::data::entities::connector_instances;
use crate::data::entities::connector_instances::NewConnectorInstanceModel;
use crate::data::repo_traits::connector_repo_errors;
use ymir::errors::Outcome;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorInstanceRepoTrait: Send + Sync {
    async fn create_instance(
        &self,
        new_instance_model: &NewConnectorInstanceModel,
    ) -> Outcome<connector_instances::Model>;

    async fn get_instance_by_id(
        &self,
        instance_id: &String,
    ) -> Outcome<Option<connector_instances::Model>>;

    async fn get_instance_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<Option<connector_instances::Model>>;

    async fn get_instances_by_distribution(
        &self,
        distribution_id: &String,
    ) -> Outcome<Option<connector_instances::Model>>;

    async fn delete_instance_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<()>;

    async fn delete_instance_by_id(&self, instance_id: &String) -> Outcome<()>;
}
