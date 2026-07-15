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

#![allow(unused)]

/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::cache::factory_redis::CatalogAgentCacheForRedis;
use crate::data::factory_sql::CatalogAgentRepoForSql;
use crate::entities::catalogs::catalogs::CatalogEntities;
use crate::entities::data_services::data_services::DataServiceEntities;
use crate::entities::datasets::datasets::DatasetEntities;
use crate::entities::distributions::distributions::DistributionEntities;
use crate::entities::odrl_policies::odrl_policies::OdrlPolicyEntities;
use crate::entities::policy_templates::policy_templates::PolicyTemplateEntities;
use crate::grpc::api::catalog_agent::catalog_entity_service_server::CatalogEntityServiceServer;
use crate::grpc::api::catalog_agent::data_service_entity_service_server::DataServiceEntityServiceServer;
use crate::grpc::api::catalog_agent::dataset_entity_service_server::DatasetEntityServiceServer;
use crate::grpc::api::catalog_agent::distribution_entity_service_server::DistributionEntityServiceServer;
use crate::grpc::api::catalog_agent::odrl_policy_entity_service_server::OdrlPolicyEntityServiceServer;
use crate::grpc::api::catalog_agent::policy_template_entity_service_server::PolicyTemplateEntityServiceServer;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::catalogs::CatalogEntityGrpc;
use crate::grpc::data_services::DataServiceEntityGrpc;
use crate::grpc::datasets::DatasetEntityGrpc;
use crate::grpc::distributions::DistributionEntityGrpc;
use crate::grpc::odrl_policies::OdrlPolicyEntityGrpc;
use crate::grpc::policy_templates::PolicyTemplateEntityGrpc;
use crate::http::catalogs::CatalogEntityRouter;
use crate::http::data_services::DataServiceEntityRouter;
use crate::http::datasets::DatasetEntityRouter;
use crate::http::distributions::DistributionEntityRouter;
use crate::http::odrl_policies::OdrlOfferEntityRouter;
use crate::http::policy_templates::PolicyTemplateEntityRouter;
use common::config::services::CatalogConfig;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait};
use sea_orm::Database;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

pub struct CatalogGrpcWorker {}

impl CatalogGrpcWorker {
    pub async fn spawn(
        config: &CatalogConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let router = Self::create_root_grpc_router(&config, vault.clone()).await?;

        let port = config.common().get_internal_port(HostType::Http);
        let addr = format!("0.0.0.0:{}", port);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy("Error listening on the socket", Some(Box::new(e))))?;
        let incoming = TcpListenerStream::new(listener);
        tracing::info!("GRPC Catalog Service running on {}", addr);

        let token = token.clone();
        let handle = tokio::spawn(async move {
            let server = router.serve_with_incoming_shutdown(incoming, async move {
                token.cancelled().await;
                tracing::info!("GRPC Service received shutdown signal, draining connections...");
            });
            match server.await {
                Ok(_) => tracing::info!("GRPC Service stopped successfully"),
                Err(e) => tracing::error!("GRPC Service crashed: {}", e),
            }
        });

        Ok(handle)
    }
    pub async fn create_root_grpc_router(
        config: &CatalogConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<tonic::transport::server::Router> {
        // conn
        let db_connection = vault.get_db_connection(config.common()).await?;
        let cache_connection_url = config.get_full_cache_url();
        let redis_client = redis::Client::open(cache_connection_url)
            .map_err(|e| Errors::crazy("Error creating Redis client", Some(Box::new(e))))?;
        let redis_connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection failed");

        // repo
        let catalog_agent_cache =
            Arc::new(CatalogAgentCacheForRedis::create_repo(redis_connection));
        let catalog_agent_repo =
            Arc::new(CatalogAgentRepoForSql::create_repo(db_connection.clone()));

        // entities
        let catalog_controller_service = Arc::new(CatalogEntities::new(
            catalog_agent_repo.clone(),
            catalog_agent_cache.clone(),
        ));
        let catalog_router = CatalogEntityGrpc::new(catalog_controller_service.clone());
        let data_services_controller_service = Arc::new(DataServiceEntities::new(
            catalog_agent_repo.clone(),
            catalog_agent_cache.clone(),
        ));
        let data_services_router =
            DataServiceEntityGrpc::new(data_services_controller_service.clone());
        let datasets_controller_service = Arc::new(DatasetEntities::new(
            catalog_agent_repo.clone(),
            catalog_agent_cache.clone(),
        ));
        let datasets_router = DatasetEntityGrpc::new(datasets_controller_service.clone());
        let distributions_controller_service = Arc::new(DistributionEntities::new(
            catalog_agent_repo.clone(),
            catalog_agent_cache.clone(),
        ));
        let distributions_router =
            DistributionEntityGrpc::new(distributions_controller_service.clone());
        let odrl_offer_controller_service = Arc::new(OdrlPolicyEntities::new(
            catalog_agent_repo.clone(),
            catalog_agent_cache.clone(),
        ));
        let odrl_offer_router = OdrlPolicyEntityGrpc::new(odrl_offer_controller_service.clone());
        let policy_templates_controller_service =
            Arc::new(PolicyTemplateEntities::new(catalog_agent_repo.clone()));
        let policy_templates_router =
            PolicyTemplateEntityGrpc::new(policy_templates_controller_service.clone());

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC server", Some(Box::new(e))))?;

        let router = Server::builder()
            .add_service(reflection_service)
            .add_service(CatalogEntityServiceServer::new(catalog_router))
            .add_service(DataServiceEntityServiceServer::new(data_services_router))
            .add_service(DatasetEntityServiceServer::new(datasets_router))
            .add_service(DistributionEntityServiceServer::new(distributions_router))
            .add_service(OdrlPolicyEntityServiceServer::new(odrl_offer_router))
            .add_service(PolicyTemplateEntityServiceServer::new(
                policy_templates_router,
            ));

        Ok(router)
    }
}
