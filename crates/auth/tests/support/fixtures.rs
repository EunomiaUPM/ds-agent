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

//! Config of the dev provider, read by the modules for the agent's own host.

use std::sync::Arc;

use common::config::services::SsiAuthConfig;
use common::config::types::traits::ConfigLoader;

/// The `ssi_auth` section of `static/environment/config/dev/dev.provider.yaml`.
pub fn config() -> Arc<SsiAuthConfig> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../static/environment/config/dev/dev.provider.yaml"
    );
    Arc::new(SsiAuthConfig::load(path).expect("dev provider config"))
}
