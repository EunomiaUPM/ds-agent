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
use ymir::config::types::ConnectionConfig;

use super::VaultSelector;

struct Flags(ConnectionConfig);

impl Flags {
    fn new(is_prod: bool, is_vault_real: bool) -> Self {
        Self(ConnectionConfig {
            is_prod,
            is_vault_real,
            has_tls_proxy: false,
        })
    }
}

impl ConnectionConfigTrait for Flags {
    fn connection(&self) -> &ConnectionConfig {
        &self.0
    }
}

/// A production config on the fake vault is rejected before any vault is built.
#[test]
fn production_on_fake_vault_is_rejected() {
    assert!(VaultSelector::check(&Flags::new(true, false)).is_err());
    assert!(VaultSelector::select(&Flags::new(true, false)).is_err());
}

/// Development may use either vault, and production the real one.
#[test]
fn other_combinations_pass_the_check() {
    assert!(VaultSelector::check(&Flags::new(false, false)).is_ok());
    assert!(VaultSelector::check(&Flags::new(false, true)).is_ok());
    assert!(VaultSelector::check(&Flags::new(true, true)).is_ok());
}
