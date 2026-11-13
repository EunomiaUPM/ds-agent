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

//! In-process L1 read-through cache over an underlying `EntityCacheTrait`.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dataplane::cache::L1DataplaneTransferCache;
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use serde_json::json;
use urn::Urn;
use ymir::cache::{EntityCacheTrait, LookupCacheTrait};
use ymir::errors::Outcome;

struct CountingCache {
    map: Mutex<HashMap<Urn, DataplaneTransferDto>>,
    main_map: Mutex<HashMap<String, DataplaneTransferDto>>,
    get_single_calls: Arc<AtomicUsize>,
    get_main_calls: Arc<AtomicUsize>,
}

impl CountingCache {
    fn new(get_single_calls: Arc<AtomicUsize>) -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            main_map: Mutex::new(HashMap::new()),
            get_single_calls,
            get_main_calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn with_main_counter(get_single_calls: Arc<AtomicUsize>, get_main_calls: Arc<AtomicUsize>) -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            main_map: Mutex::new(HashMap::new()),
            get_single_calls,
            get_main_calls,
        }
    }
}

#[async_trait::async_trait]
impl LookupCacheTrait<DataplaneTransferDto> for CountingCache {
    async fn get_by_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _limit: Option<u64>,
        _page: Option<u64>,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        Ok(vec![])
    }

    async fn add_to_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _child_id: &Urn,
        _score: f64,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn remove_from_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _child_id: &Urn,
    ) -> Outcome<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl EntityCacheTrait<DataplaneTransferDto> for CountingCache {
    async fn get_single(&self, id: &Urn) -> Outcome<Option<DataplaneTransferDto>> {
        self.get_single_calls.fetch_add(1, Ordering::Relaxed);
        Ok(self.map.lock().unwrap().get(id).cloned())
    }

    async fn set_single(&self, id: &Urn, model: &DataplaneTransferDto) -> Outcome<()> {
        self.map.lock().unwrap().insert(id.clone(), model.clone());
        Ok(())
    }

    async fn delete_single(&self, id: &Urn) -> Outcome<()> {
        self.map.lock().unwrap().remove(id);
        Ok(())
    }

    async fn get_main(&self, key: &str) -> Outcome<Option<DataplaneTransferDto>> {
        self.get_main_calls.fetch_add(1, Ordering::Relaxed);
        Ok(self.main_map.lock().unwrap().get(key).cloned())
    }

    async fn set_main(&self, key: &str, id: &Urn, model: &DataplaneTransferDto) -> Outcome<()> {
        self.main_map
            .lock()
            .unwrap()
            .insert(key.to_string(), model.clone());
        self.set_single(id, model).await
    }

    async fn get_collection(
        &self,
        _limit: Option<u64>,
        _page: Option<u64>,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        Ok(vec![])
    }

    async fn add_to_collection(&self, _id: &Urn, _score: f64) -> Outcome<()> {
        Ok(())
    }

    async fn remove_from_collection(&self, _id: &Urn) -> Outcome<()> {
        Ok(())
    }

    async fn get_batch(&self, _ids: &[Urn]) -> Outcome<Vec<DataplaneTransferDto>> {
        Ok(vec![])
    }
}

fn dto(id: &Urn, state: TransferState) -> DataplaneTransferDto {
    DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            id: id.to_string(),
            transfer_process_id: "urn:transfer-process:1".to_string(),
            role: TransferRole::Provider,
            interaction_mode: InteractionMode::Pull,
            state,
            connector_instance_id: None,
            ingress_config: json!({}),
            egress_config: json!({}),
            flow_control: None,
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }
}

#[tokio::test]
async fn l1_cache_serves_hits_until_ttl_or_local_invalidation() {
    let id = Urn::from_str("urn:dataplane-transfer:l1-test").unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let inner = CountingCache::new(calls.clone());
    inner.set_single(&id, &dto(&id, TransferState::Started)).await.unwrap();

    let cache = L1DataplaneTransferCache::with_ttl(inner, Duration::from_millis(60));

    // First read misses L1 and populates it; second read hits L1 without touching inner cache.
    let first = cache.get_single(&id).await.unwrap().unwrap();
    let second = cache.get_single(&id).await.unwrap().unwrap();
    assert_eq!(first.inner.state, TransferState::Started);
    assert_eq!(second.inner.state, TransferState::Started);
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    // Local write updates L1 immediately without causing another inner read.
    cache.set_single(&id, &dto(&id, TransferState::Stopped)).await.unwrap();
    let updated = cache.get_single(&id).await.unwrap().unwrap();
    assert_eq!(updated.inner.state, TransferState::Stopped);
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    // Local delete evicts L1 immediately and delegates the miss to inner.
    cache.delete_single(&id).await.unwrap();
    assert!(cache.get_single(&id).await.unwrap().is_none());
    assert_eq!(calls.load(Ordering::Relaxed), 2);

    // After TTL expiry, reads fall back to inner again.
    let reloaded = dto(&id, TransferState::Started);
    cache.set_single(&id, &reloaded).await.unwrap();
    assert_eq!(
        cache.get_single(&id).await.unwrap().unwrap().inner.state,
        TransferState::Started
    );
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    tokio::time::sleep(Duration::from_millis(75)).await;
    assert_eq!(
        cache.get_single(&id).await.unwrap().unwrap().inner.state,
        TransferState::Started
    );
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn l1_cache_serves_get_main_hits_and_indexes_write() {
    let id = Urn::from_str("urn:dataplane-transfer:l1-main-1").unwrap();
    let main_key = "process:urn:transfer-process:1";
    let single_calls = Arc::new(AtomicUsize::new(0));
    let main_calls = Arc::new(AtomicUsize::new(0));
    let inner = CountingCache::with_main_counter(single_calls, main_calls.clone());
    inner
        .set_main(main_key, &id, &dto(&id, TransferState::Started))
        .await
        .unwrap();

    let cache = L1DataplaneTransferCache::with_ttl(inner, Duration::from_millis(80));

    // First `get_main` misses L1 and warms it; second `get_main` hits L1 with 0 extra inner calls.
    let first = cache.get_main(main_key).await.unwrap().unwrap();
    let second = cache.get_main(main_key).await.unwrap().unwrap();
    assert_eq!(first.inner.id, id.to_string());
    assert_eq!(second.inner.state, TransferState::Started);
    assert_eq!(main_calls.load(Ordering::Relaxed), 1);

    // Local `set_single` refreshes L1 AND indexes `process:{transfer_process_id}` for `get_main`.
    cache
        .set_single(&id, &dto(&id, TransferState::Stopped))
        .await
        .unwrap();
    let after_write = cache.get_main(main_key).await.unwrap().unwrap();
    assert_eq!(after_write.inner.state, TransferState::Stopped);
    assert_eq!(main_calls.load(Ordering::Relaxed), 1);

    // Deleting the transfer evicts both L1 single and L1 main index entries.
    cache.delete_single(&id).await.unwrap();
    let _ = cache.get_main(main_key).await.unwrap();
    assert_eq!(main_calls.load(Ordering::Relaxed), 2);
}
