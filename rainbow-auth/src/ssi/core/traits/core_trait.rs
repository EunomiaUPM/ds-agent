/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use rainbow_common::config::services::SsiAuthConfig;
use ymir::core_traits::CoreWalletTrait;

use crate::ssi::core::traits::{
    CoreBusinessTrait, CoreGaiaSelfIssuerTrait, CoreGateKeeperTrait, CoreMateTrait,
    CoreOnboarderTrait, CoreVcRequesterTrait, CoreVerifierTrait
};

pub trait AuthCoreTrait:
    CoreOnboarderTrait
    + CoreWalletTrait
    + CoreVcRequesterTrait
    + CoreMateTrait
    + CoreGaiaSelfIssuerTrait
    + CoreVerifierTrait
    + CoreBusinessTrait
    + CoreVcRequesterTrait
    + CoreGateKeeperTrait
    + Send
    + Sync
    + 'static
{
    fn is_gaia_active(&self) -> bool;
    fn is_wallet_active(&self) -> bool;
    fn config(&self) -> Arc<SsiAuthConfig>;
}
