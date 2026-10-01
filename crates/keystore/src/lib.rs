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

//! Keystore: versioned parameters and secrets used by connectors and services.
//!
//! Secrets live in the database, or in Vault when a real Vault is configured. Other crates read
//! them in-process through [`SecretStore`] and [`ParameterStore`]; [`KeystoreModule`] also serves
//! them, together with the application config, under `{api}/keystore`. Its events use the
//! `keystore:` prefix.
//!
//! Modules: [`entities`] (keys, entries, versions, secret values), [`services`], [`data`],
//! `http`, [`setup`].

pub mod data;
pub mod entities;
pub(crate) mod http;
pub mod services;
pub mod setup;

pub const EVENT_DOMAIN: &str = "keystore";
pub const EVENT_PREFIX: &str = "keystore:";

pub use data::sea_orm::migrations::get_keystore_migrations;
pub use entities::entry::{Entry, SecretEntry};
pub use entities::key::{Key, KeyPrefix};
pub use entities::secret_value::SecretValue;
pub use services::parameters::ParameterStore;
pub use services::secrets::SecretStore;
pub use setup::KeystoreModule;
