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

//! Shared fixtures for catalog-agent service isolation tests: scopes, URNs and a no-op cache.

#![allow(dead_code)]

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use catalog_agent::cache::factory_trait::MockCatalogAgentCacheTrait;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::cache::EntityCacheTrait;
use common::cache::LookupCacheTrait;
use urn::Urn;
use ymir::errors::Outcome;

pub fn admin_scope() -> AccessScope {
    AccessScope::from_role(RbacRole::Admin, "admin-tenant")
}

pub fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, tenant)
}

pub fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, tenant)
}

pub fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:00000000-0000-0000-0000-{n:012}")).unwrap()
}

/// Cache adapter that discards every write and never hits, so services run without Redis.
pub struct NoopCache<D>(PhantomData<D>);

impl<D> NoopCache<D> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D> Default for NoopCache<D> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<D: Send + Sync + 'static> LookupCacheTrait<D> for NoopCache<D> {
    async fn get_by_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _limit: Option<u64>,
        _page: Option<u64>,
    ) -> Outcome<Vec<D>> {
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
impl<D: Send + Sync + 'static> EntityCacheTrait<D> for NoopCache<D> {
    async fn get_single(&self, _id: &Urn) -> Outcome<Option<D>> {
        Ok(None)
    }

    async fn set_single(&self, _id: &Urn, _model: &D) -> Outcome<()> {
        Ok(())
    }

    async fn delete_single(&self, _id: &Urn) -> Outcome<()> {
        Ok(())
    }

    async fn get_main(&self) -> Outcome<Option<D>> {
        Ok(None)
    }

    async fn set_main(&self, _id: &Urn, _model: &D) -> Outcome<()> {
        Ok(())
    }

    async fn get_collection(&self, _limit: Option<u64>, _page: Option<u64>) -> Outcome<Vec<D>> {
        Ok(vec![])
    }

    async fn add_to_collection(&self, _id: &Urn, _score: f64) -> Outcome<()> {
        Ok(())
    }

    async fn remove_from_collection(&self, _id: &Urn) -> Outcome<()> {
        Ok(())
    }

    async fn get_batch(&self, _ids: &Vec<Urn>) -> Outcome<Vec<D>> {
        Ok(vec![])
    }
}

/// Cache factory whose entity caches are all no-op.
pub fn noop_cache_factory() -> Arc<MockCatalogAgentCacheTrait> {
    let mut cache = MockCatalogAgentCacheTrait::new();
    cache
        .expect_get_catalog_cache()
        .returning(|| Arc::new(NoopCache::new()));
    cache
        .expect_get_dataservice_cache()
        .returning(|| Arc::new(NoopCache::new()));
    cache
        .expect_get_dataset_cache()
        .returning(|| Arc::new(NoopCache::new()));
    cache
        .expect_get_distribution_cache()
        .returning(|| Arc::new(NoopCache::new()));
    cache
        .expect_get_odrl_offer_cache()
        .returning(|| Arc::new(NoopCache::new()));
    Arc::new(cache)
}
