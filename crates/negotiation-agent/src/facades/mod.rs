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

//! The negotiation agent's published contract (`common::facades::negotiation`) served
//! in-process, and its public references; the catalog it consumes is the contract in
//! `common::facades::catalog`.

pub mod negotiation_facade;
pub mod public_refs;

pub use negotiation_facade::local::NegotiationLocalFacade;
pub use public_refs::local::NegotiationPublicResolver;
