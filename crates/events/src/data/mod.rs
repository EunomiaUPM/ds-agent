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

pub mod factory;
pub mod in_memory;
pub mod migrations;
pub mod repo;
pub mod sea_orm;

pub use factory::DataFactory;
pub use in_memory::{InMemoryDataFactory, InMemoryEventBusRepo};
pub use migrations::get_events_migrations;
pub use sea_orm::{SeaOrmDataFactory, SeaOrmEventBusRepo};

// Backward-compatible module alias for legacy entity references
pub mod entities {
    pub use crate::data::sea_orm::orm::*;
}
