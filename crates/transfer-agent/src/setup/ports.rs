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

//! Ports transfer consumes from other agents: in-process in the monolith, over HTTP standalone.

use std::sync::Arc;

use catalog_agent::services::datasets::DatasetServiceTrait;
use catalog_agent::services::distributions::DistributionServiceTrait;
use common::config::services::TransferConfig;
use common::config::services::traits::TransferConfigTrait;
use common::config::types::traits::CommonConfigTrait;
use common::facades::AuthPorts;
use common::module_loader::root_context::RootContext;
use connector::ConnectorInstanceFacadeTrait;
use dataplane::setup::{DataplaneModule, DataplanePorts};
use negotiation_agent::services::agreement::AgreementServiceTrait;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use crate::protocols::dsp::facades::catalog_facade::CatalogFacadeTrait;
use crate::protocols::dsp::facades::catalog_facade::local::CatalogLocalFacade;
use crate::protocols::dsp::facades::catalog_facade::remote::CatalogRemoteFacade;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::facades::dataplane_facade::local::DataPlaneLocalFacade;
use crate::protocols::dsp::facades::negotiation_facade::NegotiationFacadeTrait;
use crate::protocols::dsp::facades::negotiation_facade::local::NegotiationLocalFacade;
use crate::protocols::dsp::facades::negotiation_facade::remote::NegotiationRemoteFacade;

#[derive(Clone)]
pub struct TransferPorts {
    pub(crate) auth: AuthPorts,
    pub(crate) negotiation: Arc<dyn NegotiationFacadeTrait>,
    pub(crate) catalog: Arc<dyn CatalogFacadeTrait>,
    pub(crate) dataplane: DataplanePort,
}

/// The composed dataplane: the facade the DSP pipeline drives over its manager, and the
/// module transfer registers to serve its routes.
#[derive(Clone)]
pub(crate) struct DataplanePort {
    pub(crate) facade: Arc<dyn DataPlaneFacadeTrait>,
    pub(crate) module: DataplaneModule,
}

impl TransferPorts {
    /// Microservices: every agent is reached through its API with the service token.
    pub async fn remote(config: &TransferConfig, root: &RootContext) -> Outcome<Self> {
        let client = root.service_client.clone();
        Ok(Self {
            auth: AuthPorts::remote(config.ssi_auth(), root),
            negotiation: Arc::new(NegotiationRemoteFacade::new(
                config.contracts(),
                client.clone(),
            )),
            catalog: Arc::new(CatalogRemoteFacade::new(config.catalog(), client)),
            dataplane: Self::dataplane(config, root, &DataplanePorts::remote(config, root)).await?,
        })
    }

    /// Monolith: the owning agents' services, reached in-process. `service_tenant` is the
    /// service client's tenant, impersonated by cross-tenant lookups.
    #[allow(clippy::too_many_arguments)]
    pub async fn local(
        config: &TransferConfig,
        root: &RootContext,
        auth: AuthPorts,
        agreements: Arc<dyn AgreementServiceTrait>,
        datasets: Arc<dyn DatasetServiceTrait>,
        distributions: Arc<dyn DistributionServiceTrait>,
        connector: Arc<dyn ConnectorInstanceFacadeTrait>,
        service_tenant: String,
    ) -> Outcome<Self> {
        let dataplane_ports = DataplanePorts::local(connector.clone());
        Ok(Self {
            auth,
            negotiation: Arc::new(NegotiationLocalFacade::new(agreements, service_tenant)),
            catalog: Arc::new(CatalogLocalFacade::new(datasets, distributions, connector)),
            dataplane: Self::dataplane(config, root, &dataplane_ports).await?,
        })
    }

    pub fn auth(&self) -> &AuthPorts {
        &self.auth
    }

    /// The dataplane always runs inside transfer; only its connector lookups follow the mode.
    /// Its proxy is mounted on this process's own HTTP host.
    async fn dataplane(
        config: &TransferConfig,
        root: &RootContext,
        ports: &DataplanePorts,
    ) -> Outcome<DataplanePort> {
        let module = DataplaneModule::compose(config, root, ports).await?;
        Ok(DataplanePort {
            facade: Arc::new(DataPlaneLocalFacade::new(
                module.local_manager(),
                config.common().get_host(HostType::Http),
            )),
            module,
        })
    }
}
