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

//! Shared primitives used across the connector domain.
//!
//! | Module | Contents |
//! |---|---|
//! | [`secret_management`] | [`SecretSource`] and [`SecretString`] for credential storage |
//!
//! [`SecretSource`]: secret_management::SecretSource
//! [`SecretString`]: secret_management::SecretString

pub(crate) mod secret_management;

pub mod parameter_mutator {
    // Deprecated or removed. Use ParameterResolverBehavior instead.
}
