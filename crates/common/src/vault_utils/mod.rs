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

//! Picks the vault backend from the config: the real Vault client or the in-memory fake.
//!
//! The boot calls it once and keeps the result in the root context. A production config must
//! use the real Vault: the fake keeps secrets in clear.

use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::vault_rs::RealVaultService;
use ymir::services::vault::VaultService;

/// Builds the [`VaultService`] a config asks for.
pub struct VaultSelector;

impl VaultSelector {
    /// The real Vault client when `is_vault_real`, the fake otherwise; production requires the real one.
    pub fn select(config: &impl ConnectionConfigTrait) -> Outcome<VaultService> {
        Self::check(config)?;
        Ok(if config.is_vault_real() {
            VaultService::Real(RealVaultService::new()?)
        } else {
            VaultService::Fake(FakeVaultService::new()?)
        })
    }

    /// Fails for a production config that would run on the fake vault.
    pub fn check(config: &impl ConnectionConfigTrait) -> Outcome<()> {
        if config.is_prod() && !config.is_vault_real() {
            return Err(Errors::security(
                "is_prod requires is_vault_real: the fake vault stores secrets in clear",
                None,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
