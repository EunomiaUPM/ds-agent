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

//! Proxy configurator for Provider `PUSH` transfers backed by `ProtocolSpec::Kafka`.
//!
//! Because the dataplane actively pulls records from the Kafka cluster via `KafkaRelayWorker`,
//! no inbound HTTP listener is exposed (`DataplaneProxyIngress::NoOp`), while egress targets the
//! Consumer's `DataAddress` HTTP sink (`DataplaneProxyEgress::HttpProxy`).

use ymir::errors::Outcome;

use crate::engine::dataplane_drivers::DriverProxyConfiguratorTrait;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxy, DataplaneProxyEgress, DataplaneProxyIngress,
};
use crate::errors::DataplaneError;

#[derive(Debug)]
pub struct KafkaProviderPushConfigurator;

impl KafkaProviderPushConfigurator {
    fn configure_egress(&self, context: &DataplaneContext) -> Outcome<DataplaneProxyEgress> {
        if let Some(dataplane_address) = context.forward_dataplane_address() {
            Ok(DataplaneProxyEgress::HttpProxy {
                path: dataplane_address.endpoint.clone(),
                token_type: dataplane_address.authorization_type.clone(),
                token: dataplane_address.authorization.clone(),
            })
        } else {
            Err(DataplaneError::MissingTransferContext {
                detail: "forward dataplane address".to_string(),
            }
            .into())
        }
    }
}

#[async_trait::async_trait]
impl DriverProxyConfiguratorTrait for KafkaProviderPushConfigurator {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "kafka-broker"))]
    async fn configure_proxy(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let mut proxy = DataplaneProxy::new();
        proxy.set_ingress(DataplaneProxyIngress::NoOp);
        proxy.set_egress(self.configure_egress(context)?);
        let mut new_context = context.clone();
        new_context.set_proxy(proxy);
        Ok(new_context)
    }
}
