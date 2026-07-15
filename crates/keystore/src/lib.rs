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

pub(crate) mod data;
pub mod entities;
pub(crate) mod error;
pub(crate) mod http;
pub mod services;
pub mod setup;
pub(crate) mod utils;

pub use data::sea_orm::migrations::get_keystore_migrations;
pub use entities::entry::{Entry, SecretEntry};
pub use entities::key::{Key, KeyPrefix};
pub use entities::secret_value::SecretValue;
pub use services::parameters::ParameterStore;
pub use services::secrets::SecretStore;
pub use setup::KeystoreSetup;
