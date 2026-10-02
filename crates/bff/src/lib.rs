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

//! Gateway for the admin UI: serves the embedded single-page app and forwards its API calls
//! to the agents.
//!
//! [`GatewayHttpRouter`] mounts the app under `/admin` and [`HttpProxyDispatcher`] sends each
//! path prefix to the agent that owns it. [`BffModule`] composes the gateway into a larger
//! process and [`GatewayBoot`] runs it as its own service.
//!
//! Modules: [`gateway`] (app, discovery and router), [`proxy`], [`setup`].

pub mod gateway;
pub mod proxy;
pub mod setup;

pub use gateway::GatewayHttpRouter;
pub use proxy::HttpProxyDispatcher;
pub use setup::{AppContext, BffModule, GatewayBoot};

/// Service id used in logs and telemetry.
pub const SERVICE_NAME: &str = "gateway-agent";
/// Name shown in the boot banner.
pub const SERVICE_BIG_NAME: &str = "Gateway Agent";
