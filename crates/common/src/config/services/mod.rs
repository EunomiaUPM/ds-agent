/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

mod catalog;
mod common;
mod contracts;
mod gateway;
mod monolith;
mod ssi_auth;
pub mod traits;
mod transfer;

pub use catalog::CatalogConfig;
pub use common::CommonConfig;
pub use contracts::ContractsConfig;
pub use gateway::GatewayConfig;
pub use monolith::MonolithConfig;
pub use ssi_auth::SsiAuthConfig;
pub use transfer::TransferConfig;
