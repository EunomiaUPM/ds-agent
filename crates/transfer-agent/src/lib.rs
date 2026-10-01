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

//! Transfer agent: the control plane of the DSP transfer process.
//!
//! Peers drive it through the DSP transfer endpoints and the owner through a management API over
//! HTTP and gRPC. It checks agreements with the negotiation agent, reads datasets and connector
//! instances from the catalog, and hands the data side to the dataplane it registers.
//! [`setup::TransferAgentModule`] composes it into a larger process and [`setup::TransferBoot`]
//! runs it as its own service. Its events use the `transfers:` prefix.
//!
//! Modules: [`entities`], [`protocols`] (DSP), [`setup`], and the internal `services`, `data`,
//! `http` and `grpc`.

pub const SERVICE_NAME: &str = "transfer-agent-ref";
pub const SERVICE_BIG_NAME: &str = "Transfer Agent Ref";
pub const EVENT_DOMAIN: &str = "transfers";
pub const EVENT_PREFIX: &str = "transfers:";

mod data;
pub mod entities;
mod grpc;
mod http;
pub mod protocols;
mod services;
pub mod setup;
