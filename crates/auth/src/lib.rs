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

//! SSI authentication agent: the participant's wallet, its onboarding with peers and the
//! checks on the credentials they present.
//!
//! It requests verifiable credentials from an authority, connects to peer agents and guards
//! access with a GNAP gatekeeper backed by a verifier; issuing credentials and Gaia-X
//! self-attestation are optional. [`setup::AuthModule`] composes it into a larger process,
//! [`setup::AuthBoot`] runs it as its own service and [`facades`] serves its ports in-process.
//!
//! Modules: [`core`] (wires every service into `AuthCore`), [`modules`] (one trait per
//! capability), [`services`], [`entities`], [`types`], [`data`], [`http`], [`setup`].

/// Service id used in logs and telemetry.
pub const SERVICE_NAME: &str = "ssi-auth-agent";
/// Name shown in the boot banner.
pub const SERVICE_BIG_NAME: &str = "SSI-Auth Agent";

pub mod core;
pub mod data;
pub mod entities;
pub mod facades;
pub mod http;
pub mod modules;
pub mod services;
pub mod setup;
pub mod types;
pub mod workers;
