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

//! The dataplane engine: driver authenticators, the HTTP proxy, the per-role handlers and the
//! manager, with mocked services and connector facade.

#[path = "../support/mod.rs"]
mod support;

mod auth_api_key;
mod auth_basic;
mod auth_bearer;
mod auth_oauth;
mod handlers_consumer_pull;
mod handlers_consumer_push;
mod handlers_provider_pull;
mod handlers_provider_push;
mod handlers_strategy;
mod manager;
mod proxy_http;
