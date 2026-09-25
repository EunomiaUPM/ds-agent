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

use std::sync::Arc;

use common::auth::{AccessScope, RbacRole};
use common::boot::workers::BackgroundWorker;
use common::telemetry::TraceParent;
use events::{EventBus, EventBusTrait};
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use tracing::{info, info_span, warn, Instrument};
use ymir::errors::Outcome;

use crate::services::tenant_provisioning::TenantProvisioningServiceTrait;

/// Topic OAuth publishes when a user, and with it a tenant, is created.
const TENANT_CREATED_TOPIC: &str = "oauth:user:create";

/// Provisions every tenant born while the connector runs, reacting to OAuth events.
pub struct TenantProvisioningListener {
    bus: EventBus,
    service: Arc<dyn TenantProvisioningServiceTrait>,
}

impl TenantProvisioningListener {
    pub fn new(bus: EventBus, service: Arc<dyn TenantProvisioningServiceTrait>) -> Self {
        Self { bus, service }
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for TenantProvisioningListener {
    fn name(&self) -> &'static str {
        "tenant-provisioning"
    }

    async fn run(self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        let mut receiver = self.bus.subscribe();
        loop {
            let received = tokio::select! {
                _ = token.cancelled() => return Ok(()),
                received = receiver.recv() => received,
            };
            let envelope = match received {
                Ok(envelope) => envelope,
                Err(RecvError::Lagged(skipped)) => {
                    warn!(
                        skipped,
                        "Tenant provisioning listener lagged behind the event bus"
                    );
                    continue;
                }
                Err(RecvError::Closed) => return Ok(()),
            };
            if envelope.topic.as_str() != TENANT_CREATED_TOPIC {
                continue;
            }
            let span = info_span!(
                "event.consume",
                otel.name = %format!("{TENANT_CREATED_TOPIC} process"),
                otel.kind = "consumer",
                messaging.message.id = %envelope.id,
                tenant = %envelope.tenant_id,
            );
            TraceParent::link(&span, envelope.trace_context.as_deref());
            let tenant_id = envelope.tenant_id;
            let scope = AccessScope::from_role(RbacRole::Owner, &tenant_id);
            match self
                .service
                .provision(&scope, &tenant_id)
                .instrument(span)
                .await
            {
                Ok(_) => info!(tenant = %tenant_id, "Tenant provisioned"),
                Err(e) => warn!(tenant = %tenant_id, error = %e, "Tenant provisioning failed"),
            }
        }
    }
}
