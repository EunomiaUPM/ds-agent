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

//! Catalog-agent services with mocked repositories and no-op caches.

#[path = "../support/mod.rs"]
mod support;

mod catalogs;
mod data_services;
mod dataset_offerings;
mod datasets;
mod distributions;
mod odrl_policies;
mod peer_catalogs;
mod policy_instantiation;
mod policy_templates;
mod tenant_provisioning;
mod tenant_provisioning_listener;
