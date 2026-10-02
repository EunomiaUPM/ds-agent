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

//! Negotiation agent: the DSP contract negotiation state machine, with its offers and agreements.
//!
//! Peers drive it through the DSP negotiation endpoints and the owner through a management API
//! over HTTP and gRPC. Policies come from the catalog through a facade.
//! [`setup::NegotiationAgentModule`] composes it into a larger process and
//! [`setup::NegotiationAgentBoot`] runs it as its own service. Other crates get its migrations,
//! [`AgreementView`] and [`OfferView`] from the root; its events use the `negotiations:` prefix.
//!
//! Modules: [`entities`], [`services`], [`data`], [`protocols`] (DSP), [`http`], [`grpc`],
//! [`facades`], [`setup`].

pub mod data;
pub mod entities;
pub mod facades;
pub mod grpc;
pub mod http;
pub mod protocols;
pub mod services;
pub mod setup;

/// Domain name of the negotiation events.
pub const EVENT_DOMAIN: &str = "negotiations";
/// Topic prefix of the negotiation events.
pub const EVENT_PREFIX: &str = "negotiations:";
/// Service id used in logs and telemetry.
pub const SERVICE_NAME: &str = "negotiation-agent";
/// Name shown in the boot banner.
pub const SERVICE_BIG_NAME: &str = "Negotiation Agent Ref";

pub use data::migrations::get_negotiation_agent_migrations;
pub use services::agreement::views::AgreementView;
pub use services::offer::views::OfferView;
