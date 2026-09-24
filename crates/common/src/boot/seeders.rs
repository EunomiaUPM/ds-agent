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

//! One-off boot tasks (seeding, provisioning, cache resets) run in registration order.

use ymir::errors::Outcome;

use crate::utils::flush_redis_cache;

/// When a seeder runs relative to the workers starting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootPhase {
    /// Before any worker is spawned; nothing is being served yet.
    BeforeServe,
    /// Once HTTP/gRPC listen, for seeders that call the agent's own API.
    AfterServe,
}

#[async_trait::async_trait]
pub trait BootSeeder: Send + Sync {
    fn name(&self) -> &'static str;

    fn phase(&self) -> BootPhase {
        BootPhase::AfterServe
    }

    async fn seed(&self) -> Outcome<()>;
}

/// Empties the Redis cache so no stale entry outlives a restart.
pub struct RedisCacheFlush {
    url: String,
}

impl RedisCacheFlush {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait::async_trait]
impl BootSeeder for RedisCacheFlush {
    fn name(&self) -> &'static str {
        "redis-cache-flush"
    }

    fn phase(&self) -> BootPhase {
        BootPhase::BeforeServe
    }

    /// A cache that cannot be flushed is not fatal: log and keep booting.
    async fn seed(&self) -> Outcome<()> {
        match flush_redis_cache(&self.url).await {
            Ok(()) => tracing::info!("Redis cache at {} flushed", self.url),
            Err(e) => tracing::warn!("Failed to flush Redis at {}: {}", self.url, e),
        }
        Ok(())
    }
}
