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

use crate::CatalogDto;
use urn::Urn;

pub(crate) mod entity_cache_trait;
pub(crate) mod lookup_cache_trait;
pub(crate) mod peer_catalog_cache_trait;
pub(crate) mod redis_cache_connector_trait;
pub(crate) mod utils_trait;

const ONE_DAY_TTL: i32 = 86400;
pub(crate) const DESIRED_CACHE_TTL: i32 = ONE_DAY_TTL * 2;

pub(crate) const PEER_CATALOG_DESIRED_CACHE_TTL: i32 = ONE_DAY_TTL * 10;
