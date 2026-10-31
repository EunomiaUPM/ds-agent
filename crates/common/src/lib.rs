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

//! What the ds-agent crates share beyond ymir: config sections, DSP and JSON-LD types, facades
//! between agents, linked data and the well-known surface. Boot, errors, ownership and the other
//! cross-cutting pieces live in ymir.
//!
//! Each module explains how to use it in its own docs.
//!
//! - Config and start-up: [`config`], [`info_banner`].
//! - Agents and DSP: [`facades`], [`well_known`], [`routes`], [`dsp_common`], [`tck`].
//! - Linked data and validation: [`rdf`], [`eunomia`], [`validation`], [`flow`].
//! - HTTP and tests: [`middleware`], [`test_utils`].

pub mod config;
pub mod dsp_common;
pub mod eunomia;
pub mod facades;
pub mod flow;
pub mod info_banner;
pub mod middleware;
pub mod rdf;
pub mod routes;
pub mod tck;
pub mod test_utils;
pub mod validation;
pub mod well_known;
