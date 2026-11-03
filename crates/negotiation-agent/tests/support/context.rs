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

use common::facades::catalog::{CatalogFacadeTrait, MockCatalogFacadeTrait};
use common::facades::grants_facade::{GrantsFacadeTrait, MockGrantsFacadeTrait};
use common::facades::mates_facade::{MatesFacadeTrait, MockMatesFacadeTrait};
use common::facades::registry_facade::MockRegistryFacadeTrait;
use common::facades::AuthPorts;
use common::test_utils::config_fixtures::contracts_config_fixture;
use common::test_utils::grpc::StubTokenValidator;
use events::NoopEventSink;
use negotiation_agent::data::factory::DataFactory;
use negotiation_agent::data::repo::agreement::{AgreementRepoTrait, MockAgreementRepoTrait};
use negotiation_agent::data::repo::negotiation_message::{
    MockNegotiationMessageRepoTrait, NegotiationMessageRepoTrait,
};
use negotiation_agent::data::repo::negotiation_process::{
    MockNegotiationProcessRepoTrait, NegotiationProcessRepoTrait,
};
use negotiation_agent::data::repo::negotiation_process_identifier::{
    MockNegotiationIdentifierRepoTrait, NegotiationIdentifierRepoTrait,
};
use negotiation_agent::data::repo::offer::{MockOfferRepoTrait, OfferRepoTrait};
use negotiation_agent::services::agreement::service::AgreementService;
use negotiation_agent::services::agreement::AgreementServiceTrait;
use negotiation_agent::services::negotiation_message::service::NegotiationMessageService;
use negotiation_agent::services::negotiation_message::NegotiationMessageServiceTrait;
use negotiation_agent::services::negotiation_process::service::NegotiationProcessService;
use negotiation_agent::services::negotiation_process::NegotiationProcessServiceTrait;
use negotiation_agent::services::offer::service::OfferService;
use negotiation_agent::services::offer::OfferServiceTrait;
use negotiation_agent::setup::context::{AppContext, NegotiationServices};
use negotiation_agent::setup::NegotiationPorts;

/// The repos a test hands in.
struct StubRepos {
    processes: Arc<dyn NegotiationProcessRepoTrait>,
    messages: Arc<dyn NegotiationMessageRepoTrait>,
    identifiers: Arc<dyn NegotiationIdentifierRepoTrait>,
    offers: Arc<dyn OfferRepoTrait>,
    agreements: Arc<dyn AgreementRepoTrait>,
}

impl DataFactory for StubRepos {
    fn negotiation_process_repo(&self) -> Arc<dyn NegotiationProcessRepoTrait> {
        self.processes.clone()
    }

    fn negotiation_message_repo(&self) -> Arc<dyn NegotiationMessageRepoTrait> {
        self.messages.clone()
    }

    fn negotiation_identifier_repo(&self) -> Arc<dyn NegotiationIdentifierRepoTrait> {
        self.identifiers.clone()
    }

    fn offer_repo(&self) -> Arc<dyn OfferRepoTrait> {
        self.offers.clone()
    }

    fn agreement_repo(&self) -> Arc<dyn AgreementRepoTrait> {
        self.agreements.clone()
    }
}

/// Builds an `AppContext`; services left out are the real ones over the given repos.
pub struct TestContext {
    repos: StubRepos,
    grants: Arc<dyn GrantsFacadeTrait>,
    mates: Arc<dyn MatesFacadeTrait>,
    catalog: Arc<dyn CatalogFacadeTrait>,
    processes: Option<Arc<dyn NegotiationProcessServiceTrait>>,
    messages: Option<Arc<dyn NegotiationMessageServiceTrait>>,
    offers: Option<Arc<dyn OfferServiceTrait>>,
    agreements: Option<Arc<dyn AgreementServiceTrait>>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            repos: StubRepos {
                processes: Arc::new(MockNegotiationProcessRepoTrait::new()),
                messages: Arc::new(MockNegotiationMessageRepoTrait::new()),
                identifiers: Arc::new(MockNegotiationIdentifierRepoTrait::new()),
                offers: Arc::new(MockOfferRepoTrait::new()),
                agreements: Arc::new(MockAgreementRepoTrait::new()),
            },
            grants: Arc::new(MockGrantsFacadeTrait::new()),
            mates: Arc::new(MockMatesFacadeTrait::new()),
            catalog: Arc::new(MockCatalogFacadeTrait::new()),
            processes: None,
            messages: None,
            offers: None,
            agreements: None,
        }
    }

    pub fn process_repo(mut self, repo: impl NegotiationProcessRepoTrait + 'static) -> Self {
        self.repos.processes = Arc::new(repo);
        self
    }

    pub fn message_repo(mut self, repo: impl NegotiationMessageRepoTrait + 'static) -> Self {
        self.repos.messages = Arc::new(repo);
        self
    }

    pub fn identifier_repo(mut self, repo: impl NegotiationIdentifierRepoTrait + 'static) -> Self {
        self.repos.identifiers = Arc::new(repo);
        self
    }

    pub fn offer_repo(mut self, repo: impl OfferRepoTrait + 'static) -> Self {
        self.repos.offers = Arc::new(repo);
        self
    }

    pub fn agreement_repo(mut self, repo: impl AgreementRepoTrait + 'static) -> Self {
        self.repos.agreements = Arc::new(repo);
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

    pub fn catalog(mut self, catalog: impl CatalogFacadeTrait + 'static) -> Self {
        self.catalog = Arc::new(catalog);
        self
    }

    pub fn processes(mut self, service: impl NegotiationProcessServiceTrait + 'static) -> Self {
        self.processes = Some(Arc::new(service));
        self
    }

    pub fn messages(mut self, service: impl NegotiationMessageServiceTrait + 'static) -> Self {
        self.messages = Some(Arc::new(service));
        self
    }

    pub fn offers(mut self, service: impl OfferServiceTrait + 'static) -> Self {
        self.offers = Some(Arc::new(service));
        self
    }

    pub fn agreements(mut self, service: impl AgreementServiceTrait + 'static) -> Self {
        self.agreements = Some(Arc::new(service));
        self
    }

    pub fn build(self) -> Arc<AppContext> {
        let ports = NegotiationPorts::local(
            AuthPorts {
                mates: self.mates,
                grants: self.grants,
                registry: Arc::new(MockRegistryFacadeTrait::new()),
            },
            self.catalog,
        );
        let ctx = AppContext::bare(
            contracts_config_fixture(),
            Arc::new(self.repos),
            Arc::new(NoopEventSink),
            ports,
            Arc::new(StubTokenValidator),
        );
        let processes = self
            .processes
            .unwrap_or_else(|| Arc::new(NegotiationProcessService::new(ctx.clone())));
        let messages = self
            .messages
            .unwrap_or_else(|| Arc::new(NegotiationMessageService::new(ctx.clone())));
        let offers = self
            .offers
            .unwrap_or_else(|| Arc::new(OfferService::new(ctx.clone())));
        let agreements = self
            .agreements
            .unwrap_or_else(|| Arc::new(AgreementService::new(ctx.clone())));
        ctx.attach(NegotiationServices {
            processes,
            messages,
            offers,
            agreements,
        });
        ctx
    }
}
