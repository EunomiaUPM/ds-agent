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

//! Short-TTL in-process L1 read-through cache over an [`EntityCacheTrait`] for proxy hot paths.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use urn::Urn;
use ymir::cache::{EntityCacheTrait, LookupCacheTrait};
use ymir::errors::Outcome;

use crate::entities::dataplane_transfers::DataplaneTransferDto;

#[derive(Default)]
struct L1State {
    entries: HashMap<Urn, (Instant, DataplaneTransferDto)>,
    main_index: HashMap<String, (Instant, Urn)>,
}

/// Wraps a Redis-backed transfer cache with an L1 map invalidated synchronously on local writes.
pub struct L1DataplaneTransferCache<C> {
    inner: C,
    state: RwLock<L1State>,
    ttl: Duration,
}

impl<C> L1DataplaneTransferCache<C> {
    /// Bounds cross-replica staleness while giving the local proxy lock-free-fast hits.
    pub const DEFAULT_TTL: Duration = Duration::from_millis(750);

    pub fn new(inner: C) -> Self {
        Self::with_ttl(inner, Self::DEFAULT_TTL)
    }

    pub fn with_ttl(inner: C, ttl: Duration) -> Self {
        Self {
            inner,
            state: RwLock::new(L1State::default()),
            ttl,
        }
    }

    fn process_main_key(process_id: &str) -> String {
        format!("process:{process_id}")
    }

    fn read_entry(&self, id: &Urn) -> Option<DataplaneTransferDto> {
        let guard = self.state.read().ok()?;
        let (cached_at, dto) = guard.entries.get(id)?;
        (cached_at.elapsed() < self.ttl).then(|| dto.clone())
    }

    fn read_main(&self, key: &str) -> Option<DataplaneTransferDto> {
        let guard = self.state.read().ok()?;
        let (key_ts, id) = guard.main_index.get(key)?;
        if key_ts.elapsed() >= self.ttl {
            return None;
        }
        let (entry_ts, dto) = guard.entries.get(id)?;
        (entry_ts.elapsed() < self.ttl).then(|| dto.clone())
    }

    fn write_entry(&self, id: Urn, dto: DataplaneTransferDto, main_key: Option<&str>) {
        if let Ok(mut guard) = self.state.write() {
            let now = Instant::now();
            if guard.entries.len() > 4096 {
                let ttl = self.ttl;
                guard.entries.retain(|_, (ts, _)| ts.elapsed() < ttl);
                guard.main_index.retain(|_, (ts, _)| ts.elapsed() < ttl);
            }
            let proc_key = Self::process_main_key(&dto.inner.transfer_process_id);
            guard.main_index.insert(proc_key, (now, id.clone()));
            if let Some(key) = main_key {
                guard.main_index.insert(key.to_string(), (now, id.clone()));
            }
            guard.entries.insert(id, (now, dto));
        }
    }

    fn evict_entry(&self, id: &Urn) {
        if let Ok(mut guard) = self.state.write() {
            guard.entries.remove(id);
            guard.main_index.retain(|_, (_, target_id)| target_id != id);
        }
    }
}

#[async_trait::async_trait]
impl<C> LookupCacheTrait<DataplaneTransferDto> for L1DataplaneTransferCache<C>
where
    C: EntityCacheTrait<DataplaneTransferDto>,
{
    async fn get_by_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        self.inner
            .get_by_relation(parent_name, parent_id, limit, page)
            .await
    }

    async fn add_to_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        child_id: &Urn,
        score: f64,
    ) -> Outcome<()> {
        self.inner
            .add_to_relation(parent_name, parent_id, child_id, score)
            .await
    }

    async fn remove_from_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        child_id: &Urn,
    ) -> Outcome<()> {
        self.evict_entry(child_id);
        self.inner
            .remove_from_relation(parent_name, parent_id, child_id)
            .await
    }
}

#[async_trait::async_trait]
impl<C> EntityCacheTrait<DataplaneTransferDto> for L1DataplaneTransferCache<C>
where
    C: EntityCacheTrait<DataplaneTransferDto>,
{
    async fn get_single(&self, id: &Urn) -> Outcome<Option<DataplaneTransferDto>> {
        if let Some(cached) = self.read_entry(id) {
            return Ok(Some(cached));
        }
        let fetched = self.inner.get_single(id).await?;
        if let Some(dto) = &fetched {
            self.write_entry(id.clone(), dto.clone(), None);
        }
        Ok(fetched)
    }

    async fn set_single(&self, id: &Urn, model: &DataplaneTransferDto) -> Outcome<()> {
        self.write_entry(id.clone(), model.clone(), None);
        self.inner.set_single(id, model).await
    }

    async fn delete_single(&self, id: &Urn) -> Outcome<()> {
        self.evict_entry(id);
        self.inner.delete_single(id).await
    }

    async fn get_main(&self, key: &str) -> Outcome<Option<DataplaneTransferDto>> {
        if let Some(cached) = self.read_main(key) {
            return Ok(Some(cached));
        }
        let fetched = self.inner.get_main(key).await?;
        if let Some(dto) = &fetched {
            if let Ok(id) = Urn::from_str(&dto.inner.id) {
                self.write_entry(id, dto.clone(), Some(key));
            }
        }
        Ok(fetched)
    }

    async fn set_main(&self, key: &str, id: &Urn, model: &DataplaneTransferDto) -> Outcome<()> {
        self.write_entry(id.clone(), model.clone(), Some(key));
        self.inner.set_main(key, id, model).await
    }

    async fn get_collection(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        self.inner.get_collection(limit, page).await
    }

    async fn add_to_collection(&self, id: &Urn, score: f64) -> Outcome<()> {
        self.inner.add_to_collection(id, score).await
    }

    async fn remove_from_collection(&self, id: &Urn) -> Outcome<()> {
        self.evict_entry(id);
        self.inner.remove_from_collection(id).await
    }

    async fn get_batch(&self, ids: &[Urn]) -> Outcome<Vec<DataplaneTransferDto>> {
        self.inner.get_batch(ids).await
    }
}
