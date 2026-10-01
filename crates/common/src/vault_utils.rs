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

use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::vault_rs::RealVaultService;
use ymir::services::vault::VaultService;

/// Builds the [`VaultService`] the config asks for: the real Vault client
/// when `is_vault_real`, the in-memory fake otherwise.
pub fn vault(config: &impl ConnectionConfigTrait) -> Outcome<VaultService> {
    Ok(if config.is_vault_real() {
        VaultService::Real(RealVaultService::new()?)
    } else {
        VaultService::Fake(FakeVaultService::new()?)
    })
}
