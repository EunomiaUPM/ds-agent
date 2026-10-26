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

//! The catalog's domain: resources as RDF graphs, offers, templates and the commands and filters
//! over them. Nothing here knows about the database or HTTP.

pub mod admin_document;
pub mod commands;
pub mod dataset_offering;
pub mod errors;
pub mod filters;
pub mod graph;
pub mod jsonld;
pub mod odrl_offer;
pub mod policy_instantiation;
pub mod policy_templates;
pub mod resource;
pub mod resource_kind;
pub mod resource_set;
pub mod vocab;
