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

//! TenantProvisioningListener on a real event bus over mocked repositories: it provisions the
//! tenant of every `oauth:user:create` event and ignores other topics.

use std::sync::Arc;
use std::time::Duration;

use catalog_agent::services::tenant_provisioning::listener::TenantProvisioningListener;
use catalog_agent::services::tenant_provisioning::MockTenantProvisioningServiceTrait;
use common::oauth::RoleTrait;
use common::boot::workers::BackgroundWorker;
use events::data::repo::{
    MockEventDeadLetterRepo, MockEventDeliveryRepo, MockEventStoreRepo, MockEventSubscriptionRepo,
};
use events::{EventBus, EventEnvelope, RetryPolicy, Topic};
use serde_json::json;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;
use ymir::errors::Errors;

/// Bus that stores every event and has no webhook subscriptions.
fn bus() -> EventBus {
    let mut events = MockEventStoreRepo::new();
    events.expect_insert_event().returning(|_| Ok(()));
    let mut subscriptions = MockEventSubscriptionRepo::new();
    subscriptions
        .expect_get_matching_subscriptions()
        .returning(|_, _| Ok(vec![]));
    EventBus::new(
        Arc::new(events),
        Arc::new(subscriptions),
        Arc::new(MockEventDeliveryRepo::new()),
        Arc::new(MockEventDeadLetterRepo::new()),
        RetryPolicy::default(),
        16,
    )
}

fn event(tenant: &str, topic: &str) -> EventEnvelope {
    EventEnvelope::new(
        tenant,
        Topic::new(topic).unwrap(),
        "oauth",
        1,
        None,
        json!({}),
    )
}

/// A user creation provisions its tenant as that tenant's owner; other topics are ignored,
/// a failed provisioning does not stop the listener, and cancelling stops it cleanly.
#[tokio::test]
async fn provisions_the_tenant_of_each_user_creation() {
    let (tx, mut rx) = unbounded_channel();
    let mut service = MockTenantProvisioningServiceTrait::new();
    service
        .expect_provision()
        .withf(|scope, tenant| scope.id() == tenant && !scope.is_root())
        .returning(move |_, tenant| {
            tx.send(tenant.to_string()).unwrap();
            Err(Errors::crazy("first attempt fails", None))
        });
    let bus = bus();
    let listener = Box::new(TenantProvisioningListener::new(
        bus.clone(),
        Arc::new(service),
    ));
    let token = CancellationToken::new();
    let running = tokio::spawn(listener.run(token.clone()));
    tokio::task::yield_now().await;

    bus.publish(event("tenant-a", "catalog:dataset:create"))
        .await
        .unwrap();
    bus.publish(event("tenant-a", "oauth:user:create"))
        .await
        .unwrap();
    bus.publish(event("tenant-b", "oauth:user:create"))
        .await
        .unwrap();

    let wait = Duration::from_secs(5);
    let first = tokio::time::timeout(wait, rx.recv())
        .await
        .unwrap()
        .unwrap();
    let second = tokio::time::timeout(wait, rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!((first.as_str(), second.as_str()), ("tenant-a", "tenant-b"));

    token.cancel();
    let stopped = tokio::time::timeout(wait, running).await.unwrap().unwrap();
    assert!(stopped.is_ok());
    assert!(
        rx.try_recv().is_err(),
        "the other topic was not provisioned"
    );
}
