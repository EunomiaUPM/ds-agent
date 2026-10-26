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

//! No cache: every projection is computed, no peer catalog is kept.

use serde_json::Value;
use ymir::errors::Outcome;

use crate::cache::CatalogCacheTrait;

pub struct NoopCatalogCache;

#[async_trait::async_trait]
impl CatalogCacheTrait for NoopCatalogCache {
    async fn generation(&self) -> u64 {
        0
    }

    async fn invalidate(&self) {}

    async fn get_projection(&self, _key: &str) -> Option<Value> {
        None
    }

    async fn put_projection(&self, _key: &str, _projection: &Value) {}

    async fn get_peer_catalog(&self, _user_id: &str, _peer_id: &str) -> Outcome<Option<Value>> {
        Ok(None)
    }

    async fn put_peer_catalog(&self, _user_id: &str, _peer_id: &str, _catalog: &Value) -> Outcome<()> {
        Ok(())
    }
}
