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

pub mod ack;
pub mod auth;
pub mod command;
pub mod context_common;
pub mod context_dsp;
pub mod context_rpc;
pub mod data_address;
mod dataplane_signal;
pub mod idempotency;
pub mod message_types;
pub mod protocol_fields;
mod rdf_extractor_dsp;
pub mod state;
pub mod state_metadata;
