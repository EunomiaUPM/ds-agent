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

//! Domain model for the connector service.
//!
//! This module defines the core entities used to represent, store, and
//! instantiate connectors.
//!
//! # Architecture Overview
//!
//! The connector system follows a "blueprint and instance" pattern:
//!
//! 1. **Connector Template** ([`connector_template`]): A reusable blueprint
//!    defining metadata, authentication, and interaction protocols. It uses
//!    `{{__PARAM__}}` placeholders for configuration.
//! 2. **Connector Instance** ([`connector_instance`]): A concrete deployment of a
//!    template. It binds values to the template's parameters, resulting in a
//!    fully resolved configuration ready for the dataplane.
//!
//! # The Resolution Pipeline
//!
//! When a template is instantiated, it goes through a multi-stage pipeline
//! managed by the [`parameters`] module:
//!
//! - **Extraction**: Discovering all placeholders referenced in the template.
//! - **Validation**: Ensuring user-supplied values match the template's requirements.
//! - **Enrichment**: Injecting system-level parameters (e.g., URNs, timestamps).
//! - **Resolution**: Performing the final interpolation, replacing all placeholders
//!   with their concrete values.
//!
//! # Sub-module overview
//!
//! | Module | Contents |
//! |---|---|
//! | [`auth_config`] | `AuthenticationConfig` and credential types |
//! | [`common`] | Shared primitives: parameter types, template wrappers, secrets |
//! | [`connector_instance`] | Instance DTOs and lifecycle management |
//! | [`connector_template`] | Template DTOs and blueprint management |
//! | [`interaction`] | `InteractionConfig` (Pull / Push) and lifecycle types |
//! | [`resource`] | Protocol specs: `HttpSpec`, `KafkaSpec`, `ProtocolSpec` |
//! | `parameters` *(private)* | The parameter extraction, enrichment, and resolution engine |

pub mod auth_config;
pub mod common;
pub mod connector_instance;
pub mod connector_template;
pub mod interaction;
pub(crate) mod parameters;
pub mod resource;
