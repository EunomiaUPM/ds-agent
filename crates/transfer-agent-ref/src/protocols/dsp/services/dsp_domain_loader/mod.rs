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

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_dsp::TransferDSPContextTyped;
use common::dsp_common::odrl::OdrlAgreement;
use ymir::errors::Outcome;

pub mod dsp_domain_loader;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait DspDomainLoaderTrait: Send + Sync {
    async fn resolve_agreement(&self, typed: &TransferDSPContextTyped) -> Outcome<OdrlAgreement>;
    async fn resolve_role_for_new(&self, typed: &TransferDSPContextTyped) -> Outcome<TransferRole>;
}
