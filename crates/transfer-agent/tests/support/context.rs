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
use common::facades::negotiation::{MockNegotiationFacadeTrait, NegotiationFacadeTrait};
use common::facades::AuthPorts;
use common::test_utils::config_fixtures::transfer_config_fixture;
use common::test_utils::grpc::StubTokenValidator;
use connector::{ConnectorInstanceFacadeTrait, MockConnectorInstanceFacadeTrait};
use events::NoopEventSink;
use transfer_agent::data::factory::DataFactory;
use transfer_agent::data::repo::transfer_message::{
    MockTransferMessageRepoTrait, TransferMessageRepoTrait,
};
use transfer_agent::data::repo::transfer_process::{
    MockTransferProcessRepoTrait, TransferProcessRepoTrait,
};
use transfer_agent::data::repo::transfer_process_identifier::{
    MockTransferIdentifierRepoTrait, TransferIdentifierRepoTrait,
};
use transfer_agent::protocols::dsp::facades::dataplane_facade::{
    DataPlaneFacadeTrait, MockDataPlaneFacadeTrait,
};
use transfer_agent::services::transfer_message::service::TransferMessageService;
use transfer_agent::services::transfer_message::TransferMessageServiceTrait;
use transfer_agent::services::transfer_process::service::TransferProcessService;
use transfer_agent::services::transfer_process::TransferProcessServiceTrait;
use transfer_agent::setup::context::{AppContext, TransferServices};
use transfer_agent::setup::TransferPorts;

/// The repos a test hands in.
struct StubRepos {
    processes: Arc<dyn TransferProcessRepoTrait>,
    messages: Arc<dyn TransferMessageRepoTrait>,
    identifiers: Arc<dyn TransferIdentifierRepoTrait>,
}

impl DataFactory for StubRepos {
    fn transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait> {
        self.processes.clone()
    }

    fn transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait> {
        self.messages.clone()
    }

    fn transfer_identifier_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait> {
        self.identifiers.clone()
    }
}

/// Builds an `AppContext`; services left out are the real ones over the given repos.
pub struct TestContext {
    repos: StubRepos,
    grants: Arc<dyn GrantsFacadeTrait>,
    mates: Arc<dyn MatesFacadeTrait>,
    negotiation: Arc<dyn NegotiationFacadeTrait>,
    catalog: Arc<dyn CatalogFacadeTrait>,
    connectors: Arc<dyn ConnectorInstanceFacadeTrait>,
    dataplane: Arc<dyn DataPlaneFacadeTrait>,
    processes: Option<Arc<dyn TransferProcessServiceTrait>>,
    messages: Option<Arc<dyn TransferMessageServiceTrait>>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            repos: StubRepos {
                processes: Arc::new(MockTransferProcessRepoTrait::new()),
                messages: Arc::new(MockTransferMessageRepoTrait::new()),
                identifiers: Arc::new(MockTransferIdentifierRepoTrait::new()),
            },
            grants: Arc::new(MockGrantsFacadeTrait::new()),
            mates: Arc::new(MockMatesFacadeTrait::new()),
            negotiation: Arc::new(MockNegotiationFacadeTrait::new()),
            catalog: Arc::new(MockCatalogFacadeTrait::new()),
            connectors: Arc::new(MockConnectorInstanceFacadeTrait::new()),
            dataplane: Arc::new(MockDataPlaneFacadeTrait::new()),
            processes: None,
            messages: None,
        }
    }

    pub fn process_repo(mut self, repo: impl TransferProcessRepoTrait + 'static) -> Self {
        self.repos.processes = Arc::new(repo);
        self
    }

    pub fn message_repo(mut self, repo: impl TransferMessageRepoTrait + 'static) -> Self {
        self.repos.messages = Arc::new(repo);
        self
    }

    pub fn identifier_repo(mut self, repo: impl TransferIdentifierRepoTrait + 'static) -> Self {
        self.repos.identifiers = Arc::new(repo);
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

    pub fn negotiation(mut self, negotiation: impl NegotiationFacadeTrait + 'static) -> Self {
        self.negotiation = Arc::new(negotiation);
        self
    }

    pub fn catalog(mut self, catalog: impl CatalogFacadeTrait + 'static) -> Self {
        self.catalog = Arc::new(catalog);
        self
    }

    pub fn connectors(mut self, connectors: impl ConnectorInstanceFacadeTrait + 'static) -> Self {
        self.connectors = Arc::new(connectors);
        self
    }

    pub fn dataplane(mut self, dataplane: impl DataPlaneFacadeTrait + 'static) -> Self {
        self.dataplane = Arc::new(dataplane);
        self
    }

    pub fn processes(mut self, service: impl TransferProcessServiceTrait + 'static) -> Self {
        self.processes = Some(Arc::new(service));
        self
    }

    pub fn messages(mut self, service: impl TransferMessageServiceTrait + 'static) -> Self {
        self.messages = Some(Arc::new(service));
        self
    }

    pub fn build(self) -> Arc<AppContext> {
        let ports = TransferPorts::stubbed(
            AuthPorts {
                mates: self.mates,
                grants: self.grants,
                registry: Arc::new(MockRegistryFacadeTrait::new()),
            },
            self.negotiation,
            self.catalog,
            self.connectors,
            self.dataplane,
        );
        let ctx = AppContext::bare(
            transfer_config_fixture(),
            Arc::new(self.repos),
            Arc::new(NoopEventSink),
            ports,
            Arc::new(StubTokenValidator),
        );
        let processes = self
            .processes
            .unwrap_or_else(|| Arc::new(TransferProcessService::new(ctx.clone())));
        let messages = self
            .messages
            .unwrap_or_else(|| Arc::new(TransferMessageService::new(ctx.clone())));
        ctx.attach(TransferServices {
            processes,
            messages,
        });
        ctx
    }
}
