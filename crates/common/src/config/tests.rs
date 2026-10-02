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

use crate::config::services::TransferConfig;
use crate::config::types::traits::{CommonConfigTrait, ConfigLoader};
use crate::config::ApplicationConfig;

const DEV_PROVIDER: &str = "../../static/environment/config/dev/dev.provider.yaml";

/// The dev provider file carries every section the monolith composes.
#[test]
fn dev_config_passes_monolith_validation() {
    let config = <ApplicationConfig as ConfigLoader>::load(DEV_PROVIDER).unwrap();

    assert!(config.transfer().is_ok());
    assert!(config.gateway().is_ok());
}

/// A config without agent sections is rejected at load, naming every missing section.
#[test]
fn missing_sections_are_reported_together() {
    let full = ApplicationConfig::load(DEV_PROVIDER).unwrap();
    let bare = ApplicationConfig::new(full.common().clone());

    let err = bare.validate_monolith().unwrap_err().reason().to_string();

    for section in ["ssi_auth", "transfer", "contracts", "catalog", "gateway"] {
        assert!(err.contains(section), "{section} missing from: {err}");
    }
    assert!(bare.transfer().is_err());
}

/// An agent file holding only its own section loads through the local fallback.
#[test]
fn agent_file_without_other_sections_falls_back_to_local_load() {
    let transfer = ApplicationConfig::load(DEV_PROVIDER)
        .unwrap()
        .transfer()
        .unwrap()
        .clone();
    let path = std::env::temp_dir().join(format!("transfer-only-{}.yaml", std::process::id()));
    std::fs::write(&path, serde_norway::to_string(&transfer).unwrap()).unwrap();

    let loaded = TransferConfig::load(path.to_str().unwrap());
    std::fs::remove_file(&path).ok();

    assert!(loaded.is_ok());
}
