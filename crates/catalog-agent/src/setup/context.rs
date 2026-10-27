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

//! What every catalog module uses: config, repos, events, cache, ports, the profile registry and
//! the services built on them.

use std::sync::{Arc, OnceLock};

use common::config::services::CatalogConfig;
use common::module_loader::root_context::RootContext;
use events::EventSinkTrait;
use ymir::services::token_validator::OauthTokenValidatorTrait;

use crate::cache::CatalogCacheTrait;
use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::profiles::ProfileRegistry;
use crate::services::catalog_import::service::CatalogImportService;
use crate::services::catalog_import::CatalogImportServiceTrait;
use crate::services::closure::service::ClosureService;
use crate::services::closure::ClosureServiceTrait;
use crate::services::dataset_offerings::service::DatasetOfferingService;
use crate::services::dataset_offerings::DatasetOfferingServiceTrait;
use crate::services::doctor::service::DoctorService;
use crate::services::doctor::DoctorServiceTrait;
use crate::services::main_catalog::service::MainCatalogService;
use crate::services::main_catalog::MainCatalogServiceTrait;
use crate::services::odrl_policies::service::OdrlPolicyService;
use crate::services::odrl_policies::OdrlPolicyServiceTrait;
use crate::services::peer_catalogs::service::PeerCatalogService;
use crate::services::peer_catalogs::PeerCatalogServiceTrait;
use crate::services::policy_instantiation::service::PolicyInstantiationService;
use crate::services::policy_instantiation::PolicyInstantiationServiceTrait;
use crate::services::policy_templates::service::PolicyTemplateService;
use crate::services::policy_templates::PolicyTemplateServiceTrait;
use crate::services::projection::service::ProjectionService;
use crate::services::projection::ProjectionServiceTrait;
use crate::services::resources::service::ResourceService;
use crate::services::resources::ResourceServiceTrait;
use crate::setup::ports::CatalogPorts;

/// The catalog agent's context; every service, router and adapter keeps it whole.
pub struct AppContext {
    pub config: Arc<CatalogConfig>,
    pub repos: Arc<dyn DataFactory>,
    pub events: Arc<dyn EventSinkTrait>,
    pub cache: Arc<dyn CatalogCacheTrait>,
    pub ports: CatalogPorts,
    pub oauth_validator: Arc<dyn OauthTokenValidatorTrait>,
    /// The profile adapters the composition registered.
    pub profiles: ProfileRegistry,
    services: OnceLock<CatalogServices>,
}

/// The agent's own services; they keep the context, so they are attached once it exists.
#[derive(Clone)]
pub struct CatalogServices {
    pub resources: Arc<dyn ResourceServiceTrait>,
    pub closures: Arc<dyn ClosureServiceTrait>,
    pub offers: Arc<dyn OdrlPolicyServiceTrait>,
    pub templates: Arc<dyn PolicyTemplateServiceTrait>,
    pub instantiation: Arc<dyn PolicyInstantiationServiceTrait>,
    pub offerings: Arc<dyn DatasetOfferingServiceTrait>,
    pub peer_catalogs: Arc<dyn PeerCatalogServiceTrait>,
    pub main_catalog: Arc<dyn MainCatalogServiceTrait>,
    pub doctor: Arc<dyn DoctorServiceTrait>,
    pub projections: Arc<dyn ProjectionServiceTrait>,
    pub imports: Arc<dyn CatalogImportServiceTrait>,
}

impl CatalogServices {
    /// The real services over the context's repos, cache and events.
    pub fn own(ctx: &Arc<AppContext>) -> Self {
        Self {
            resources: Arc::new(ResourceService::new(ctx.clone())),
            closures: Arc::new(ClosureService::new(ctx.clone())),
            offers: Arc::new(OdrlPolicyService::new(ctx.clone())),
            templates: Arc::new(PolicyTemplateService::new(ctx.clone())),
            instantiation: Arc::new(PolicyInstantiationService::new(ctx.clone())),
            offerings: Arc::new(DatasetOfferingService::new(ctx.clone())),
            peer_catalogs: Arc::new(PeerCatalogService::new(ctx.clone())),
            main_catalog: Arc::new(MainCatalogService::new(ctx.clone())),
            doctor: Arc::new(DoctorService::new(ctx.clone())),
            projections: Arc::new(ProjectionService::new(ctx.clone())),
            imports: Arc::new(CatalogImportService::new(ctx.clone())),
        }
    }
}

impl AppContext {
    /// SeaORM repos on the shared pool, the given cache, events and ports, and the own services.
    pub fn build(
        config: &CatalogConfig,
        root: &RootContext,
        cache: Arc<dyn CatalogCacheTrait>,
        events: Arc<dyn EventSinkTrait>,
        ports: CatalogPorts,
    ) -> Arc<Self> {
        let ctx = Self::bare(
            Arc::new(config.clone()),
            Arc::new(SeaOrmDataFactory::new(root.db.clone())),
            cache,
            events,
            ports,
            root.validator.clone(),
        );
        ctx.attach(CatalogServices::own(&ctx));
        ctx
    }

    /// The context without services; `attach` completes it (tests attach mocks).
    pub fn bare(
        config: Arc<CatalogConfig>,
        repos: Arc<dyn DataFactory>,
        cache: Arc<dyn CatalogCacheTrait>,
        events: Arc<dyn EventSinkTrait>,
        ports: CatalogPorts,
        oauth_validator: Arc<dyn OauthTokenValidatorTrait>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            repos,
            events,
            cache,
            ports,
            oauth_validator,
            profiles: ProfileRegistry::default(),
            services: OnceLock::new(),
        })
    }

    /// Attaches the services; only the first call counts.
    pub fn attach(&self, services: CatalogServices) {
        let _ = self.services.set(services);
    }

    pub fn resources(&self) -> &Arc<dyn ResourceServiceTrait> {
        &self.services().resources
    }

    pub fn closures(&self) -> &Arc<dyn ClosureServiceTrait> {
        &self.services().closures
    }

    pub fn offers(&self) -> &Arc<dyn OdrlPolicyServiceTrait> {
        &self.services().offers
    }

    pub fn templates(&self) -> &Arc<dyn PolicyTemplateServiceTrait> {
        &self.services().templates
    }

    pub fn instantiation(&self) -> &Arc<dyn PolicyInstantiationServiceTrait> {
        &self.services().instantiation
    }

    pub fn offerings(&self) -> &Arc<dyn DatasetOfferingServiceTrait> {
        &self.services().offerings
    }

    pub fn peer_catalogs(&self) -> &Arc<dyn PeerCatalogServiceTrait> {
        &self.services().peer_catalogs
    }

    pub fn main_catalog(&self) -> &Arc<dyn MainCatalogServiceTrait> {
        &self.services().main_catalog
    }

    pub fn doctor(&self) -> &Arc<dyn DoctorServiceTrait> {
        &self.services().doctor
    }

    pub fn projections(&self) -> &Arc<dyn ProjectionServiceTrait> {
        &self.services().projections
    }

    pub fn imports(&self) -> &Arc<dyn CatalogImportServiceTrait> {
        &self.services().imports
    }

    fn services(&self) -> &CatalogServices {
        self.services
            .get()
            .expect("services are attached before any module runs")
    }
}
