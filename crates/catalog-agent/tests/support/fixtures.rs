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

//! URNs and a cache factory whose caches are all no-op, so services run without Redis.

use std::str::FromStr;
use std::sync::Arc;

use catalog_agent::cache::factory_trait::MockCatalogAgentCacheTrait;
use common::cache::NoopCache;
use urn::Urn;

pub fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:00000000-0000-0000-0000-{n:012}")).unwrap()
}

/// Same URN as [`test_urn`], as the string a gRPC request carries.
pub fn urn(n: u32) -> String {
    test_urn(n).to_string()
}

/// Cache factory whose entity caches are all [`NoopCache`].
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
