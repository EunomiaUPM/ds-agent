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

//! Wires the catalog data layer and management services once, for every driving adapter.

use std::sync::Arc;

use crate::cache::factory_redis::CatalogAgentCacheForRedis;
use crate::data::factory_sql::CatalogAgentRepoForSql;
use crate::entities::catalogs::catalogs::CatalogEntities;
use crate::entities::catalogs::CatalogEntityTrait;
use crate::entities::data_services::data_services::DataServiceEntities;
use crate::entities::data_services::DataServiceEntityTrait;
use crate::entities::datasets::datasets::DatasetEntities;
use crate::entities::datasets::DatasetEntityTrait;
use crate::entities::distributions::distributions::DistributionEntities;
use crate::entities::distributions::DistributionEntityTrait;
use crate::entities::odrl_policies::odrl_policies::OdrlPolicyEntities;
use crate::entities::odrl_policies::OdrlPolicyEntityTrait;
use crate::entities::policy_templates::policy_templates::PolicyTemplateEntities;
use crate::entities::policy_templates::PolicyTemplateEntityTrait;
use common::auth::OauthTokenValidator;
use common::config::services::CatalogConfig;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait};
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

#[derive(Clone)]
pub struct AppContext {
    pub catalog_svc: Arc<dyn CatalogEntityTrait>,
    pub data_service_svc: Arc<dyn DataServiceEntityTrait>,
    pub dataset_svc: Arc<dyn DatasetEntityTrait>,
    pub distribution_svc: Arc<dyn DistributionEntityTrait>,
    pub odrl_policy_svc: Arc<dyn OdrlPolicyEntityTrait>,
    pub policy_template_svc: Arc<dyn PolicyTemplateEntityTrait>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
}

impl AppContext {
    pub async fn build_with_bus(
        config: &CatalogConfig,
        vault: &VaultService,
        event_bus: Option<events::EventBus>,
    ) -> Outcome<Self> {
        // Shared infrastructure
        let db = vault.get_db_connection(config.common()).await?;
        let redis = redis::Client::open(config.get_full_cache_url())
            .map_err(|e| Errors::crazy("Error creating Redis client", Some(Box::new(e))))?
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Errors::crazy("Redis connection failed", Some(Box::new(e))))?;
        let cache = Arc::new(CatalogAgentCacheForRedis::create_repo(redis));
        let repo = Arc::new(CatalogAgentRepoForSql::create_repo(db.clone()));

        // Oauth module validator
        let oauth_validator: Arc<dyn OauthTokenValidator> =
            oauth::setup::composition::OAuthSetup::new()
                .build_token_service(config.common().clone().into(), db);

        // Domain services
        let catalog_svc = Arc::new(
            CatalogEntities::new(repo.clone(), cache.clone()).with_event_bus(event_bus.clone()),
        );
        let data_service_svc = Arc::new(
            DataServiceEntities::new(repo.clone(), cache.clone()).with_event_bus(event_bus.clone()),
        );
        let dataset_svc = Arc::new(
            DatasetEntities::new(repo.clone(), cache.clone()).with_event_bus(event_bus.clone()),
        );
        let distribution_svc = Arc::new(
            DistributionEntities::new(repo.clone(), cache.clone())
                .with_event_bus(event_bus.clone()),
        );
        let odrl_policy_svc = Arc::new(
            OdrlPolicyEntities::new(repo.clone(), cache.clone()).with_event_bus(event_bus.clone()),
        );
        let policy_template_svc =
            Arc::new(PolicyTemplateEntities::new(repo).with_event_bus(event_bus));

        Ok(Self {
            catalog_svc,
            data_service_svc,
            dataset_svc,
            distribution_svc,
            odrl_policy_svc,
            policy_template_svc,
            oauth_validator,
        })
    }
}
