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

//! An `AppContext` over mocks; every double left out is a bare mock, so touching it fails.

use std::sync::Arc;

use catalog_agent::cache::{CatalogCacheTrait, NoopCatalogCache};
use catalog_agent::data::factory::DataFactory;
use catalog_agent::data::repo::odrl_offer::{MockOdrlOfferRepoTrait, OdrlOfferRepoTrait};
use catalog_agent::data::repo::policy_template::{
    MockPolicyTemplateRepoTrait, PolicyTemplateRepoTrait,
};
use catalog_agent::data::repo::resource::{MockResourceRepoTrait, ResourceRepoTrait};
use catalog_agent::data::repo::schema::{MockSchemaRepoTrait, SchemaRepoTrait};
use catalog_agent::profiles::dcat3::Dcat3;
use catalog_agent::profiles::dcat_ap::DcatAp;
use catalog_agent::profiles::dsp::DspProfileAdapter;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::services::catalog_import::CatalogImportServiceTrait;
use catalog_agent::services::closure::ClosureServiceTrait;
use catalog_agent::services::doctor::DoctorServiceTrait;
use catalog_agent::services::odrl_policies::OdrlPolicyServiceTrait;
use catalog_agent::services::policy_composition::PolicyCompositionServiceTrait;
use catalog_agent::services::policy_templates::PolicyTemplateServiceTrait;
use catalog_agent::services::projection::ProjectionServiceTrait;
use catalog_agent::services::resources::ResourceServiceTrait;
use catalog_agent::services::schemas::SchemaServiceTrait;
use catalog_agent::setup::context::{AppContext, CatalogServices};
use catalog_agent::setup::CatalogPorts;
use common::facades::grants_facade::{GrantsFacadeTrait, MockGrantsFacadeTrait};
use common::facades::mates_facade::{MatesFacadeTrait, MockMatesFacadeTrait};
use common::facades::AuthPorts;
use common::test_utils::config_fixtures::catalog_config_fixture;
use common::test_utils::grpc::StubTokenValidator;
use events::NoopEventSink;

/// The repos a test hands in.
struct StubRepos {
    resources: Arc<dyn ResourceRepoTrait>,
    offers: Arc<dyn OdrlOfferRepoTrait>,
    templates: Arc<dyn PolicyTemplateRepoTrait>,
    schemas: Arc<dyn SchemaRepoTrait>,
}

impl DataFactory for StubRepos {
    fn resource_repo(&self) -> Arc<dyn ResourceRepoTrait> {
        self.resources.clone()
    }

    fn odrl_offer_repo(&self) -> Arc<dyn OdrlOfferRepoTrait> {
        self.offers.clone()
    }

    fn policy_template_repo(&self) -> Arc<dyn PolicyTemplateRepoTrait> {
        self.templates.clone()
    }

    fn schema_repo(&self) -> Arc<dyn SchemaRepoTrait> {
        self.schemas.clone()
    }
}

/// Builds an `AppContext` with the three profiles registered; services left out are the real
/// ones over the given repos.
pub struct TestContext {
    repos: StubRepos,
    cache: Arc<dyn CatalogCacheTrait>,
    grants: Arc<dyn GrantsFacadeTrait>,
    mates: Arc<dyn MatesFacadeTrait>,
    resources: Option<Arc<dyn ResourceServiceTrait>>,
    closures: Option<Arc<dyn ClosureServiceTrait>>,
    offers: Option<Arc<dyn OdrlPolicyServiceTrait>>,
    templates: Option<Arc<dyn PolicyTemplateServiceTrait>>,
    composition: Option<Arc<dyn PolicyCompositionServiceTrait>>,
    doctor: Option<Arc<dyn DoctorServiceTrait>>,
    projections: Option<Arc<dyn ProjectionServiceTrait>>,
    imports: Option<Arc<dyn CatalogImportServiceTrait>>,
    schemas: Option<Arc<dyn SchemaServiceTrait>>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            repos: StubRepos {
                resources: Arc::new(MockResourceRepoTrait::new()),
                offers: Arc::new(MockOdrlOfferRepoTrait::new()),
                templates: Arc::new(MockPolicyTemplateRepoTrait::new()),
                schemas: Arc::new(MockSchemaRepoTrait::new()),
            },
            cache: Arc::new(NoopCatalogCache),
            grants: Arc::new(MockGrantsFacadeTrait::new()),
            mates: Arc::new(MockMatesFacadeTrait::new()),
            resources: None,
            closures: None,
            offers: None,
            templates: None,
            composition: None,
            doctor: None,
            projections: None,
            imports: None,
            schemas: None,
        }
    }

    pub fn resource_repo(mut self, repo: impl ResourceRepoTrait + 'static) -> Self {
        self.repos.resources = Arc::new(repo);
        self
    }

    pub fn offer_repo(mut self, repo: impl OdrlOfferRepoTrait + 'static) -> Self {
        self.repos.offers = Arc::new(repo);
        self
    }

    pub fn template_repo(mut self, repo: impl PolicyTemplateRepoTrait + 'static) -> Self {
        self.repos.templates = Arc::new(repo);
        self
    }

    pub fn schema_repo(mut self, repo: impl SchemaRepoTrait + 'static) -> Self {
        self.repos.schemas = Arc::new(repo);
        self
    }

    pub fn schemas(mut self, service: impl SchemaServiceTrait + 'static) -> Self {
        self.schemas = Some(Arc::new(service));
        self
    }

    pub fn cache(mut self, cache: impl CatalogCacheTrait + 'static) -> Self {
        self.cache = Arc::new(cache);
        self
    }

    pub fn grants(mut self, grants: impl GrantsFacadeTrait + 'static) -> Self {
        self.grants = Arc::new(grants);
        self
    }

    pub fn mates(mut self, mates: impl MatesFacadeTrait + 'static) -> Self {
        self.mates = Arc::new(mates);
        self
    }

    pub fn resources(mut self, service: impl ResourceServiceTrait + 'static) -> Self {
        self.resources = Some(Arc::new(service));
        self
    }

    pub fn closures(mut self, service: impl ClosureServiceTrait + 'static) -> Self {
        self.closures = Some(Arc::new(service));
        self
    }

    pub fn offers(mut self, service: impl OdrlPolicyServiceTrait + 'static) -> Self {
        self.offers = Some(Arc::new(service));
        self
    }

    pub fn templates(mut self, service: impl PolicyTemplateServiceTrait + 'static) -> Self {
        self.templates = Some(Arc::new(service));
        self
    }

    pub fn composition(mut self, service: impl PolicyCompositionServiceTrait + 'static) -> Self {
        self.composition = Some(Arc::new(service));
        self
    }

    pub fn doctor(mut self, service: impl DoctorServiceTrait + 'static) -> Self {
        self.doctor = Some(Arc::new(service));
        self
    }

    pub fn projections(mut self, service: impl ProjectionServiceTrait + 'static) -> Self {
        self.projections = Some(Arc::new(service));
        self
    }

    pub fn imports(mut self, service: impl CatalogImportServiceTrait + 'static) -> Self {
        self.imports = Some(Arc::new(service));
        self
    }

    pub fn build(self) -> Arc<AppContext> {
        let ports = CatalogPorts::local(AuthPorts {
            mates: self.mates,
            grants: self.grants,
        });
        let ctx = AppContext::bare(
            catalog_config_fixture(),
            Arc::new(self.repos),
            self.cache,
            Arc::new(NoopEventSink),
            ports,
            Arc::new(StubTokenValidator),
        );
        let mut services = CatalogServices::own(&ctx);
        if let Some(resources) = self.resources {
            services.resources = resources;
        }
        if let Some(closures) = self.closures {
            services.closures = closures;
        }
        if let Some(offers) = self.offers {
            services.offers = offers;
        }
        if let Some(templates) = self.templates {
            services.templates = templates;
        }
        if let Some(composition) = self.composition {
            services.composition = composition;
        }
        if let Some(doctor) = self.doctor {
            services.doctor = doctor;
        }
        if let Some(projections) = self.projections {
            services.projections = projections;
        }
        if let Some(imports) = self.imports {
            services.imports = imports;
        }
        if let Some(schemas) = self.schemas {
            services.schemas = schemas;
        }
        ctx.attach(services);
        ctx.profiles.register(Arc::new(DspProfileAdapter::new()));
        ctx.profiles
            .register(Arc::new(DcatAdapter::<Dcat3>::default()));
        ctx.profiles
            .register(Arc::new(DcatAdapter::<DcatAp>::default()));
        ctx
    }
}
