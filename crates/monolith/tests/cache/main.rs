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

//! Catalog agent Redis caches against the server `REDIS_URL` points at. Every key a test writes
//! is new to that run, so the dev data in the same Redis is left alone.

mod catalog_cache;
mod dataservice_cache;

/// Async connection to the Redis at `REDIS_URL`.
async fn redis() -> redis::aio::MultiplexedConnection {
    let url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must point at a Redis server for integration tests");
    redis::Client::open(url)
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .expect("connect to REDIS_URL")
}
