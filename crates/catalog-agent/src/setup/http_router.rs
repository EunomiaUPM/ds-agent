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

//! Legacy HTTP plane of the catalog agent, built as one router until it becomes a module.

use crate::cache::factory_redis::CatalogAgentCacheForRedis;
use crate::data::factory_sql::CatalogAgentRepoForSql;
use crate::http::catalogs::CatalogEntityRouter;
use crate::http::data_services::DataServiceEntityRouter;
use crate::http::dataset_offerings::DatasetOfferingRouter;
use crate::http::datasets::DatasetEntityRouter;
use crate::http::distributions::DistributionEntityRouter;
use crate::http::odrl_policies::OdrlOfferEntityRouter;
use crate::http::peer_catalog::PeerCatalogEntityRouter;
use crate::http::policy_templates::PolicyTemplateEntityRouter;
use crate::http::tenants::TenantRouter;
use crate::protocols::dsp::CatalogDSP;
use crate::protocols::protocol::ProtocolPluginTrait;
use crate::services::catalogs::service::CatalogService;
use crate::services::data_services::service::DataServiceService;
use crate::services::dataset_offerings::service::DatasetOfferingService;
use crate::services::datasets::service::DatasetService;
use crate::services::distributions::service::DistributionService;
use crate::services::odrl_policies::service::OdrlPolicyService;
use crate::services::peer_catalogs::service::PeerCatalogService;
use crate::services::policy_instantiation::service::PolicyInstantiationService;
use crate::services::policy_templates::service::PolicyTemplateService;
use crate::services::tenant_provisioning::service::TenantProvisioningService;
use axum::Router;
use common::config::services::traits::CatalogConfigTrait;
use common::config::services::CatalogConfig;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait, MinKnownConfigTrait};
use common::facades::ssi_auth_facade::mates_facade::MatesFacadeService;
use common::http_client::HttpClient;
use common::module_loader::root_context::RootContext;
use connector::ConnectorSetup;
use std::sync::Arc;
use ymir::config::traits::{ApiConfigTrait, ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};

pub async fn create_root_http_router(
    config: &CatalogConfig,
    root: &RootContext,
    event_bus: Option<events::EventBus>,
) -> Outcome<Router> {
    // ROOT Dependency Injection
    let db_connection = root.db.clone();
    let config = Arc::new(config.clone());
    let cache_connection_url = config.get_full_cache_url();
    let redis_client = redis::Client::open(cache_connection_url)
        .map_err(|e| Errors::crazy("Not able to connect to Redis client", Some(Box::new(e))))?;
    let redis_connection = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Redis connection failed");
    let http_client = Arc::new(HttpClient::new(20, 3));

    // repo
    let catalog_agent_cache = Arc::new(CatalogAgentCacheForRedis::create_repo(redis_connection));
    let catalog_agent_repo = Arc::new(CatalogAgentRepoForSql::create_repo(db_connection.clone()));

    // facades
    let ssi_auth_config = Arc::new(config.ssi_auth().clone());
    let service_client = root.service_client.clone();
    let mates_facade = Arc::new(MatesFacadeService::new(
        ssi_auth_config.clone(),
        service_client.clone(),
    ));

    // entities
    let catalog_controller_service = Arc::new(
        CatalogService::new(catalog_agent_repo.clone(), catalog_agent_cache.clone())
            .with_event_bus(event_bus.clone()),
    );
    let catalog_router =
        CatalogEntityRouter::new(catalog_controller_service.clone(), config.clone());
    let data_services_controller_service = Arc::new(
        DataServiceService::new(catalog_agent_repo.clone(), catalog_agent_cache.clone())
            .with_event_bus(event_bus.clone()),
    );
    let data_services_router =
        DataServiceEntityRouter::new(data_services_controller_service.clone(), config.clone());
    let datasets_controller_service = Arc::new(
        DatasetService::new(catalog_agent_repo.clone(), catalog_agent_cache.clone())
            .with_event_bus(event_bus.clone()),
    );
    let datasets_router =
        DatasetEntityRouter::new(datasets_controller_service.clone(), config.clone());
    let distributions_controller_service = Arc::new(
        DistributionService::new(catalog_agent_repo.clone(), catalog_agent_cache.clone())
            .with_event_bus(event_bus.clone()),
    );
    let distributions_router =
        DistributionEntityRouter::new(distributions_controller_service.clone(), config.clone());
    let odrl_offer_controller_service = Arc::new(
        OdrlPolicyService::new(catalog_agent_repo.clone(), catalog_agent_cache.clone())
            .with_event_bus(event_bus.clone()),
    );
    let odrl_offer_router =
        OdrlOfferEntityRouter::new(odrl_offer_controller_service.clone(), config.clone());

    let policy_templates_controller_service = Arc::new(
        PolicyTemplateService::new(catalog_agent_repo.clone()).with_event_bus(event_bus.clone()),
    );
    let policy_engine_service = Arc::new(PolicyInstantiationService::new(
        odrl_offer_controller_service.clone(),
        policy_templates_controller_service.clone(),
    ));
    let policy_templates_router = PolicyTemplateEntityRouter::new(
        policy_templates_controller_service.clone(),
        policy_engine_service.clone(),
        config.clone(),
    );
    let dataset_offering_router =
        DatasetOfferingRouter::new(Arc::new(DatasetOfferingService::new(
            catalog_controller_service.clone(),
            data_services_controller_service.clone(),
            datasets_controller_service.clone(),
            distributions_controller_service.clone(),
            odrl_offer_controller_service.clone(),
        )));
    let tenant_provisioning = Arc::new(TenantProvisioningService::new(
        catalog_controller_service.clone(),
        data_services_controller_service.clone(),
        mates_facade.clone(),
        format!(
            "{}/dsp/current",
            config.contracts().get_host(HostType::Http)
        ),
    ));
    let tenant_router = TenantRouter::new(tenant_provisioning);
    let peer_catalog_service = Arc::new(PeerCatalogService::new(
        catalog_agent_cache.clone(),
        mates_facade.clone(),
    ));
    let peer_catalog_router = PeerCatalogEntityRouter::new(peer_catalog_service.clone());

    // connector module
    let connector_router =
        ConnectorSetup::new().build_control_router(&config, root, event_bus.clone());

    let validator = root.validator.clone();

    // dsp
    let dsp_router = CatalogDSP::new(
        catalog_controller_service.clone(),
        data_services_controller_service.clone(),
        datasets_controller_service.clone(),
        odrl_offer_controller_service.clone(),
        distributions_controller_service.clone(),
        peer_catalog_service.clone(),
        mates_facade.clone(),
        config.clone(),
        service_client,
        validator.clone(),
    )
    .build_router()
    .await?;

    let catalog_router_str = format!("{}/catalog-agent", config.common().get_api_version());
    let connector_router_str = format!("{}/connector", config.common().get_api_version());

    let management_router = Router::new()
        .nest(
            format!("{}/catalogs", catalog_router_str.as_str()).as_str(),
            catalog_router.router(),
        )
        .nest(
            format!("{}/data-services", catalog_router_str.as_str()).as_str(),
            data_services_router.router(),
        )
        .nest(
            format!("{}/datasets", catalog_router_str.as_str()).as_str(),
            datasets_router.router(),
        )
        .nest(
            format!("{}/distributions", catalog_router_str.as_str()).as_str(),
            distributions_router.router(),
        )
        .nest(
            format!("{}/odrl-policies", catalog_router_str.as_str()).as_str(),
            odrl_offer_router.router(),
        )
        .nest(
            format!("{}/policy-templates", catalog_router_str.as_str()).as_str(),
            policy_templates_router.router(),
        )
        .nest(
            format!("{}/peer-catalogs", catalog_router_str.as_str()).as_str(),
            peer_catalog_router.router(),
        )
        .nest(
            format!("{}/dataset-offerings", catalog_router_str.as_str()).as_str(),
            dataset_offering_router.router(),
        )
        .nest(
            format!("{}/tenants", catalog_router_str.as_str()).as_str(),
            tenant_router.router(),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            validator,
            common::auth::http::AuthHttpMiddleware::run,
        ));

    let router = Router::new()
        .merge(management_router)
        .nest("/dsp/current/catalog", dsp_router)
        .nest(connector_router_str.as_str(), connector_router);

    Ok(router)
}
