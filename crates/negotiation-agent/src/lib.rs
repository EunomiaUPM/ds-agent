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

pub mod data;
pub mod entities;
pub(crate) mod facades;
pub mod grpc;
pub mod http;
pub mod protocols;
pub mod services;
pub mod setup;

pub const EVENT_DOMAIN: &str = "negotiations";
pub const EVENT_PREFIX: &str = "negotiations:";
pub const SERVICE_NAME: &str = "negotiation-agent";
pub const SERVICE_BIG_NAME: &str = "Negotiation Agent Ref";

pub use data::migrations::get_negotiation_agent_migrations;
pub use services::agreement::views::AgreementView;
pub use services::offer::views::OfferView;
