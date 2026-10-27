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

//! The management API (internal: users): every kind of resource as JSON-LD, plus offers,
//! templates, offerings, the catalogs fetched from peers and catalog import and export.

pub mod catalog_exchange;
pub mod common;
pub mod dataset_offerings;
pub mod odrl_policies;
pub mod peer_catalogs;
pub mod policy_templates;
pub mod resources;
