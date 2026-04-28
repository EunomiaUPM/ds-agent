#![allow(unused)]
/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

pub(crate) mod cache;
pub(crate) mod data;
pub(crate) mod entities;
pub(crate) mod errors;
pub(crate) mod facades;
pub mod http;
pub mod setup;
pub mod testing_proxy;

pub use data::migrations::get_dataplane_migrations;
pub use entities::dataplane_manager::dataplane_commands::{
    DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneCommandResponse
};
pub use entities::dataplane_manager::dataplane_manager::DataplaneManager;
pub use entities::dataplane_manager::DataplaneAddress;
pub use entities::dataplane_transfers::DataplaneTransfersEntitiesTrait;

#[cfg(test)]
pub(crate) mod test_fixtures;
