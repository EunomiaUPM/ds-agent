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

//! Loading DCAT documents into the catalog: validated with the profile's SHACL shapes, planned
//! against what is stored and written whole, or not at all.

pub mod service;
pub mod views;

use common::rdf::RdfSyntax;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::commands::ImportCatalogCommand;
use crate::services::catalog_import::views::ImportReport;

/// Importing documents into the catalog.
#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogImportServiceTrait: Send + Sync {
    /// Validates `body` and plans it; unless it is a dry run or anything keeps it out, writes it.
    async fn import(
        &self,
        user: &UserInfo,
        syntax: RdfSyntax,
        body: &[u8],
        command: &ImportCatalogCommand,
    ) -> Outcome<ImportReport>;
}
