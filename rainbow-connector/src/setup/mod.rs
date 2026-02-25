use crate::data::factory_sql::ConnectorRepoForSql;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::entities::connector_instance::connector_instance::ConnectorInstanceEntitiesService;
use crate::entities::connector_instance::ConnectorInstanceTrait;
use crate::entities::connector_template::connector_template::ConnectorTemplateEntitiesService;
use crate::facades::distribution_resolver_facade::data_service_resolver_facade::DistributionFacadeServiceForConnector;
use crate::http::connector_instance::ConnectorInstanceRouter;
use crate::http::connector_template::ConnectorTemplateRouter;
use axum::Router;
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::config::types::traits::CommonConfigTrait;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;

pub struct ConnectorSetup {}
impl ConnectorSetup {
    pub fn new() -> Self {
        ConnectorSetup {}
    }

    pub async fn get_connector_repo(
        &self,
        config: &CatalogConfig,
        vault: Arc<VaultService>,
    ) -> Arc<dyn ConnectorRepoTrait> {
        let db_connection = vault.get_db_connection(config.common()).await;
        Arc::new(ConnectorRepoForSql::create_repo(db_connection))
    }

    pub async fn get_connector_instance_entity<C: CommonConfigTrait + Send + Sync>(
        &self,
        config: &C,
        vault: Arc<VaultService>,
        http_client: Arc<HttpClient>,
    ) -> Arc<dyn ConnectorInstanceTrait> {
        let db_connection = vault.get_db_connection(config.common()).await;
        let connector_repo: Arc<dyn ConnectorRepoTrait> = Arc::new(ConnectorRepoForSql::create_repo(db_connection));
        let distribution_facade = Arc::new(DistributionFacadeServiceForConnector::new(
            config,
            http_client,
        ));
        Arc::new(ConnectorInstanceEntitiesService::new(
            connector_repo,
            distribution_facade,
        ))
    }

    pub async fn build_control_router(
        &self,
        config: &CatalogConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let connector_repo = self.get_connector_repo(config, vault.clone()).await;
        let config_arc = Arc::new(config.clone());
        let http_client = Arc::new(HttpClient::new(3, 1));

        let distribution_facade = Arc::new(DistributionFacadeServiceForConnector::new(
            config,
            http_client.clone(),
        ));

        let connector_template_service =
            Arc::new(ConnectorTemplateEntitiesService::new(connector_repo.clone()));
        let connector_template_router =
            ConnectorTemplateRouter::new(connector_template_service.clone(), config_arc.clone())
                .router();
        let connector_instance_service = Arc::new(ConnectorInstanceEntitiesService::new(
            connector_repo.clone(),
            distribution_facade.clone(),
        ));
        let connector_instance_router =
            ConnectorInstanceRouter::new(connector_instance_service.clone()).router();
        Router::new()
            .nest("/templates", connector_template_router)
            .nest("/instances", connector_instance_router)
    }
}
