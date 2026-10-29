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

//! JSON Schemas that describe data: inferred from sample documents, checked against the
//! 2020-12 meta-schema, and samples checked against them. Nothing is fetched over the network.

mod check;
mod inference;

pub use check::{JsonSchemaCheck, SampleReport, SchemaIssue};
pub use inference::JsonSchemaInference;

/// The dialect every schema written here declares.
pub const JSON_SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Media type of a JSON Schema document.
pub const JSON_SCHEMA_MEDIA_TYPE: &str = "application/schema+json";

#[cfg(test)]
mod tests;
