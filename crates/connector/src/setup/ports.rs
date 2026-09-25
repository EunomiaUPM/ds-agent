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

//! Ports the connector consumes: always in-process, since it runs inside the catalog agent.

use std::sync::Arc;

use crate::facades::catalog_facade::CatalogFacadeTrait;

#[derive(Clone)]
pub struct ConnectorPorts {
    pub(crate) catalog: Arc<dyn CatalogFacadeTrait>,
}

impl ConnectorPorts {
    /// `catalog` is the hosting catalog's in-process adapter.
    pub fn local(catalog: Arc<dyn CatalogFacadeTrait>) -> Self {
        Self { catalog }
    }
}
