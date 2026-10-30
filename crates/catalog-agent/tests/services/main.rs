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

//! Catalog-agent use cases over mocked repositories: resources, closures, the doctor, offers,
//! offerings, templates and their composition, the main catalog, projections and imports.

#[path = "../support/mod.rs"]
mod support;

mod catalog_facade;
mod catalog_import;
mod closure;
mod dataset_offerings;
mod doctor;
mod main_catalog;
mod odrl_policies;
mod policy_composition;
mod policy_templates;
mod projection;
mod resources;
mod schemas;
