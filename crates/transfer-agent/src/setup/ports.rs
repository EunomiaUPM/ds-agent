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

use std::str::FromStr;
use std::sync::Arc;

use common::config::services::TransferConfig;
use common::config::services::traits::TransferConfigTrait;
use common::facades::AuthPorts;
use common::facades::catalog::CatalogFacadeTrait;
use common::facades::catalog::remote::CatalogRemoteFacade;
use common::facades::negotiation::NegotiationFacadeTrait;
use common::facades::negotiation::remote::NegotiationRemoteFacade;
use ymir::module_loader::root_context::RootContext;
use connector::{ConnectorInstanceDto, ConnectorInstanceFacadeTrait, ConnectorInstanceRemoteFacade};
use dataplane::setup::{DataplaneModule, DataplanePorts};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::facades::dataplane_facade::local::DataPlaneLocalFacade;

/// Ports the transfer agent consumes: auth, the negotiation and catalog contracts, the
/// connector instances the catalog hosts, and its own dataplane.
#[derive(Clone)]
pub struct TransferPorts {
    pub auth: AuthPorts,
    pub negotiation: Arc<dyn NegotiationFacadeTrait>,
    pub catalog: Arc<dyn CatalogFacadeTrait>,
    pub connectors: Arc<dyn ConnectorInstanceFacadeTrait>,
    /// The hooks the DSP flows drive over the dataplane manager.
    pub dataplane: Arc<dyn DataPlaneFacadeTrait>,
    /// The dataplane module transfer registers; none when the dataplane is stubbed.
    dataplane_module: Option<DataplaneModule>,
}

impl TransferPorts {
    /// Microservices: every agent is reached through its API.
    pub async fn remote(config: &TransferConfig, root: &RootContext) -> Outcome<Self> {
        let module =
            DataplaneModule::compose(config, root, &DataplanePorts::remote(config, root)).await?;
        Ok(Self {
            auth: AuthPorts::remote(config.ssi_auth()),
            negotiation: Arc::new(NegotiationRemoteFacade::new(config.contracts())),
            catalog: Arc::new(CatalogRemoteFacade::new(config.catalog())),
            connectors: Arc::new(ConnectorInstanceRemoteFacade::new(config.catalog())),
            dataplane: Arc::new(DataPlaneLocalFacade::new(module.local_manager())),
            dataplane_module: Some(module),
        })
    }

    /// Monolith: the owning agents' published contracts, served in-process.
    pub async fn local(
        config: &TransferConfig,
        root: &RootContext,
        auth: AuthPorts,
        negotiation: Arc<dyn NegotiationFacadeTrait>,
        catalog: Arc<dyn CatalogFacadeTrait>,
        connectors: Arc<dyn ConnectorInstanceFacadeTrait>,
    ) -> Outcome<Self> {
        // The dataplane always runs inside transfer; only its connector lookups follow the mode.
        let module =
            DataplaneModule::compose(config, root, &DataplanePorts::local(connectors.clone()))
                .await?;
        Ok(Self {
            auth,
            negotiation,
            catalog,
            connectors,
            dataplane: Arc::new(DataPlaneLocalFacade::new(module.local_manager())),
            dataplane_module: Some(module),
        })
    }

    /// Ports over the given facades and no dataplane module, for tests.
    pub fn stubbed(
        auth: AuthPorts,
        negotiation: Arc<dyn NegotiationFacadeTrait>,
        catalog: Arc<dyn CatalogFacadeTrait>,
        connectors: Arc<dyn ConnectorInstanceFacadeTrait>,
        dataplane: Arc<dyn DataPlaneFacadeTrait>,
    ) -> Self {
        Self {
            auth,
            negotiation,
            catalog,
            connectors,
            dataplane,
            dataplane_module: None,
        }
    }

    pub fn dataplane_module(&self) -> Option<DataplaneModule> {
        self.dataplane_module.clone()
    }

    /// The connector instance serving `target` in `format`: its dataset, then the distribution
    /// in that format, then the instance behind it.
    pub async fn connector_for(&self, target: &Urn, format: &str) -> Outcome<ConnectorInstanceDto> {
        let dataset = self.catalog.get_dataset(target).await?;
        let dataset_id = Urn::from_str(&dataset.id)?;
        let distribution = self.catalog.get_distribution_by_format(&dataset_id, format).await?;
        let distribution_id = Urn::from_str(&distribution.id)?;
        self.connectors
            .get_instance_by_distribution(&distribution_id)
            .await?
            // A distribution nobody bound a connector to cannot be transferred: the peer gets a
            // reason it can act on, not an internal error.
            .ok_or_else(|| {
                Errors::missing_resource(
                    distribution_id.to_string(),
                    format!("distribution {distribution_id} has no connector to serve it"),
                    None,
                )
            })
    }
}
