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

//! Use cases of the DSP transfer process: the inbound and outbound flows, the persistence they
//! share, the validators, auto-start and the TCK scenarios.

pub mod auto_start;
pub mod dataplane_signals;
pub mod dsp_flow;
pub mod dsp_persistence;
pub mod rpc_flow;
pub mod tck;
pub mod validator;
